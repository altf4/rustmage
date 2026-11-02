//! Layer-two control effects. The object controller is a derived cache; baselines
//! and resolution-created effects remain owned independently of their sources.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ControlBase {
    pub object: ObjectRef,
    pub controller: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ControlEffect {
    pub object: ObjectRef,
    pub controller: Player,
    pub timestamp: u64,
    pub expires_turn: Option<u32>,
    pub expires_after_next_turn: Option<NextTurnControlExpiry>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ControlState {
    pub bases: Vec<ControlBase>,
    pub effects: Vec<ControlEffect>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
struct AppliedControl {
    object: ObjectRef,
    controller: Player,
    source: Option<ObjectRef>,
    /// Fixed control only while the captured player controls this exact source.
    source_condition: Option<(ObjectRef, Player)>,
    timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    pub(crate) fn controls_attached(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group
                .fragments
                .iter()
                .any(|fragment| matches!(fragment, StaticAbilityFragment::ControlAttached))
        })
    }
}
/// Re-evaluate using the supplied actual or speculative object facts. Layer-six
/// removal cannot undo an effect that already started in layer two (CR613.6).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn controllers<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
) -> Vec<ControlBase> {
    let object = |id| world.object(id).clone();
    let Some(state) = world.controls() else {
        return Vec::new();
    };
    let present = |r: ObjectRef| {
        let o = object(r.card);
        o.zone == Zone::Battlefield && o.generation == r.generation
    };
    let mut result: Vec<_> = state
        .bases
        .iter()
        .copied()
        .filter(|row| present(row.object))
        .collect();
    let mut effects: Vec<_> = state
        .effects
        .iter()
        .filter(|row| present(row.object))
        .map(|row| AppliedControl {
            object: row.object,
            controller: row.controller,
            source: None,
            source_condition: None,
            timestamp: row.timestamp,
        })
        .collect();
    if let Some(roles) = world.roles() {
        effects.extend(
            roles
                .source_linked_effects
                .iter()
                .filter(|row| {
                    matches!(
                        row.kind,
                        SourceLinkedEffectKind::Control
                            | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
                    ) && present(row.source)
                        && present(row.recipient)
                })
                .map(|row| AppliedControl {
                    object: row.recipient,
                    controller: row.controller,
                    source: None,
                    source_condition: (row.kind
                        == SourceLinkedEffectKind::ControlAndCombatLockWhileControlled)
                        .then_some((row.source, row.controller)),
                    timestamp: row.timestamp,
                }),
        );
    }
    for &id in world.attachment_sources() {
        let aura = object(id);
        let source = ObjectRef {
            card: id,
            generation: aura.generation,
        };
        if aura.zone != Zone::Battlefield || !aura.card.controls_attached() {
            continue;
        }
        let Some(target) = aura.attached_to.filter(|&r| present(r)) else {
            continue;
        };
        let timestamp = world
            .roles()
            .and_then(|r| r.ability_timestamps.iter().find(|r| r.source == source))
            .expect("registered layer-two source")
            .timestamp;
        effects.push(AppliedControl {
            object: target,
            controller: aura.controller,
            source: Some(source),
            source_condition: None,
            timestamp,
        });
    }
    effects.sort_by_key(|row| row.timestamp);
    // An Aura's effect depends on any effect changing its own controller. Apply
    // independent effects first regardless of timestamp. In a dependency loop,
    // ignore the cyclic dependencies and use timestamp order (CR613.8).
    while !effects.is_empty() {
        let controller_of = |reference: ObjectRef| {
            result
                .iter()
                .find(|row| row.object == reference)
                .map_or_else(|| object(reference.card).controller, |row| row.controller)
        };
        let produces =
            |effect: AppliedControl| effect.source.map_or(effect.controller, controller_of);
        // CR613.8a requires an actual change to what the dependent effect does.
        // A control assignment to the controller already projected is no dependency.
        // Rebuild this relation after every effect, as required by CR613.8c.
        let depends = |index: usize, other: usize| {
            index != other
                && (effects[index].source.is_some_and(|source| {
                    effects[other].object == source
                        && produces(effects[other]) != controller_of(source)
                }) || effects[index]
                    .source_condition
                    .is_some_and(|(source, expected)| {
                        effects[other].object == source
                            && (produces(effects[other]) == expected)
                                != (controller_of(source) == expected)
                    }))
        };
        let reaches = |start: usize, goal: usize| {
            let mut seen = vec![false; effects.len()];
            let mut pending = vec![start];
            while let Some(index) = pending.pop() {
                if seen[index] {
                    continue;
                }
                seen[index] = true;
                for next in 0..effects.len() {
                    if depends(index, next) {
                        if next == goal {
                            return true;
                        }
                        pending.push(next);
                    }
                }
            }
            false
        };
        let chosen = (0..effects.len())
            .find(|&index| {
                !(0..effects.len()).any(|other| depends(index, other) && !reaches(other, index))
            })
            .unwrap_or(0);
        let controller = produces(effects[chosen]);
        let applies = effects[chosen]
            .source_condition
            .is_none_or(|(source, expected)| controller_of(source) == expected);
        let effect = effects.remove(chosen);
        if !applies {
            continue;
        }
        if let Some(row) = result.iter_mut().find(|row| row.object == effect.object) {
            row.controller = controller;
        }
    }
    result
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn ensure_control_base(&mut self, object: ObjectRef) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let controller = self.state.objects[object.card as usize].controller;
        let state = self.state.controls.get_or_insert_with(Default::default);
        if !state.bases.iter().any(|row| row.object == object) {
            state.bases.push(ControlBase { object, controller });
        }
    }
    pub(super) fn record_control(
        &mut self,
        object: ObjectRef,
        controller: Player,
        expires_turn: Option<u32>,
    ) {
        self.record_control_expiring(object, controller, expires_turn, None);
    }
    pub(super) fn record_control_expiring(
        &mut self,
        object: ObjectRef,
        controller: Player,
        expires_turn: Option<u32>,
        expires_after_next_turn: Option<NextTurnControlExpiry>,
    ) {
        self.ensure_control_base(object);
        let timestamp = self.next_role_timestamp();
        self.state
            .controls
            .as_mut()
            .unwrap()
            .effects
            .push(ControlEffect {
                object,
                controller,
                timestamp,
                expires_turn,
                expires_after_next_turn,
            });
        self.refresh_controllers();
    }
    pub(super) fn refresh_controllers(&mut self) {
        // Capture bases before projecting new attachment effects. The previous
        // cache still represents the world before this attachment changed.
        let targets: Vec<_> = self
            .state
            .attachment_sources
            .iter()
            .filter_map(|&id| {
                let object = &self.state.objects[id as usize];
                (object.zone == Zone::Battlefield && object.card.controls_attached())
                    .then_some(object.attached_to)
                    .flatten()
            })
            .collect();
        for &target in &targets {
            self.ensure_control_base(target);
        }
        if self.state.controls.is_none() {
            self.refresh_copy_grant_availability();
            self.refresh_enduring_stories();
            return;
        }
        for row in controllers(self) {
            self.change_controller(row.object, row.controller);
        }
        let live: Vec<_> = self
            .state
            .controls
            .as_ref()
            .unwrap()
            .bases
            .iter()
            .filter(|row| {
                self.on_battlefield(row.object.card)
                    && self.object_ref(row.object.card) == row.object
            })
            .map(|row| row.object)
            .collect();
        let linked: Vec<_> = self
            .state
            .roles
            .as_ref()
            .into_iter()
            .flat_map(|r| &r.source_linked_effects)
            .filter(|row| {
                matches!(
                    row.kind,
                    SourceLinkedEffectKind::Control
                        | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
                ) && source_linked_effects::active(self, row)
            })
            .map(|row| row.recipient)
            .collect();
        let state = self.state.controls.as_mut().unwrap();
        state.bases.retain(|row| live.contains(&row.object));
        state.effects.retain(|row| live.contains(&row.object));
        state.bases.retain(|row| {
            state
                .effects
                .iter()
                .any(|effect| effect.object == row.object)
                || targets.contains(&row.object)
                || linked.contains(&row.object)
        });
        if state.bases.is_empty() {
            self.state.controls = None;
        }
        self.refresh_copy_grant_availability();
        self.refresh_enduring_stories();
    }
    pub(super) fn control_bytes(&self) -> usize {
        self.state.controls.as_ref().map_or(0, |state| {
            std::mem::size_of::<ControlState>()
                + state.bases.len() * std::mem::size_of::<ControlBase>()
                + state.effects.len() * std::mem::size_of::<ControlEffect>()
        })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_control_exchange(&mut self, first: ObjectRef, second: ObjectRef) {
        let controllers = [
            self.state.objects[first.card as usize].controller,
            self.state.objects[second.card as usize].controller,
        ];
        if controllers[0] == controllers[1] {
            return;
        }
        self.ensure_control_base(first);
        self.ensure_control_base(second);
        let timestamp = self.next_role_timestamp();
        for (object, controller) in [(first, controllers[1]), (second, controllers[0])] {
            self.forget_temporary_control(object);
            let state = self.state.controls.as_mut().unwrap();
            state.effects.retain(|row| {
                row.object != object
                    || (row.expires_turn.is_none() && row.expires_after_next_turn.is_none())
            });
            state.effects.push(ControlEffect {
                object,
                controller,
                timestamp,
                expires_turn: None,
                expires_after_next_turn: None,
            });
        }
        self.refresh_controllers();
    }
}
