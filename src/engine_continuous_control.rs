//! Layer-two control effects. The object controller is a derived cache; baselines
//! and resolution-created effects remain owned independently of their sources.
use super::*;
use crate::cards::StaticAbilityFragment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ControlBase {
    pub object: ObjectRef,
    pub controller: Player,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ControlEffect {
    pub object: ObjectRef,
    pub controller: Player,
    pub timestamp: u64,
    pub expires_turn: Option<u32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ControlState {
    pub bases: Vec<ControlBase>,
    pub effects: Vec<ControlEffect>,
}
#[derive(Clone, Copy)]
struct AppliedControl {
    object: ObjectRef,
    controller: Player,
    source: Option<ObjectRef>,
    timestamp: u64,
}
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
            timestamp: row.timestamp,
        })
        .collect();
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
                && effects[index].source.is_some_and(|source| {
                    effects[other].object == source
                        && produces(effects[other]) != controller_of(source)
                })
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
        let effect = effects.remove(chosen);
        if let Some(row) = result.iter_mut().find(|row| row.object == effect.object) {
            row.controller = controller;
        }
    }
    result
}
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
        let state = self.state.controls.as_mut().unwrap();
        state.bases.retain(|row| live.contains(&row.object));
        state.effects.retain(|row| live.contains(&row.object));
        state.bases.retain(|row| {
            state
                .effects
                .iter()
                .any(|effect| effect.object == row.object)
                || targets.contains(&row.object)
        });
        if state.bases.is_empty() {
            self.state.controls = None;
        }
    }
    pub(super) fn control_bytes(&self) -> usize {
        self.state.controls.as_ref().map_or(0, |state| {
            std::mem::size_of::<ControlState>()
                + state.bases.len() * std::mem::size_of::<ControlBase>()
                + state.effects.len() * std::mem::size_of::<ControlEffect>()
        })
    }
}
