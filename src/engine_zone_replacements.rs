//! Typed graveyard-event replacements and rare owned competing-effect choices.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::ZoneReplacementRule;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneReplacementKind {
    RevealShuffle,
    /// The event is prevented; the object remains in its original zone.
    Prevent,
    Exile,
    /// Distinct replacement identity; only the selected source creates a reflexive packet.
    ExileReflexive {
        source: ObjectRef,
        ability: u16,
    },
    Battlefield,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ZoneRevealView {
    pub card: Card,
    pub owner: Player,
    pub from: Zone,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ZoneReplacementChoiceView {
    pub card: Card,
    pub owner: Player,
    pub from: Zone,
    pub choices: Vec<ZoneReplacementKind>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Request {
    object: ObjectRef,
    requested: Zone,
    affected: Player,
    view: ZoneReplacementChoiceView,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pick {
    object: ObjectRef,
    requested: Zone,
    effect: ZoneReplacementKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Base {
    state: Box<State>,
    bytes: usize,
    actor: Player,
    action: Action,
    picks: SharedVec<Pick>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Choice {
    base: Base,
    request: Request,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Replay {
    base: Base,
    next: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ZoneState {
    choice: Option<Box<Choice>>,
    replay: Option<Box<Replay>>,
    reveals: SharedVec<ZoneRevealView>,
    graveyard_exile_spells: SharedVec<ObjectRef>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct FrozenRules {
    pub prevent: bool,
    pub shuffle_self: bool,
    pub exile_spell: bool,
    pub discard_to_battlefield: bool,
    pub reflexive_exiles: Vec<(u16, ObjectView)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Plan {
    pub destination: Zone,
    pub prevented: bool,
    pub shuffle: bool,
    pub reflexive: Option<(u16, Box<ObjectView>)>,
}
impl ZoneState {
    // Choices and replays own checkpoints; passive reveals and object links do not.
    #[cfg(test)]
    pub(super) fn has_checkpoint(&self) -> bool {
        self.choice.is_some() || self.replay.is_some()
    }
    pub fn bytes(&self) -> usize {
        let base_bytes = |base: &Base| {
            base.bytes
                + base.picks.len() * std::mem::size_of::<Pick>()
                + match &base.action {
                    Action::Activate {
                        counter_payment, ..
                    }
                    | Action::ActivateMana {
                        counter_payment, ..
                    }
                    | Action::ActivateBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::ActivateGraveyardBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceGraveyardBorrowedMana {
                        counter_payment, ..
                    }
                    | Action::AnnounceBorrowedAbility {
                        counter_payment, ..
                    }
                    | Action::AnnounceGraveyardBorrowedAbility {
                        counter_payment, ..
                    }
                    | Action::BeginActivate {
                        counter_payment, ..
                    }
                    | Action::AnnounceActivate {
                        counter_payment, ..
                    }
                    | Action::AnnounceMana {
                        counter_payment, ..
                    } => counter_payment.owned_bytes(),
                    _ => 0,
                }
        };
        std::mem::size_of::<Self>()
            + self.reveals.len() * std::mem::size_of::<ZoneRevealView>()
            + self.graveyard_exile_spells.len() * std::mem::size_of::<ObjectRef>()
            + self.choice.as_ref().map_or(0, |c| {
                std::mem::size_of::<Choice>()
                    + base_bytes(&c.base)
                    + c.request.view.choices.len() * std::mem::size_of::<ZoneReplacementKind>()
            })
            + self
                .replay
                .as_ref()
                .map_or(0, |r| std::mem::size_of::<Replay>() + base_bytes(&r.base))
    }
}
#[cfg(test)]
thread_local! {
    pub(super) static TEST_INVALIDATE_STAGED_COST: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_FAKE_ENTRY_LIFE: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_REDIRECT_EXILE_TO_BATTLEFIELD: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_PREVENT_ZONE_MOVE: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_PREVENT_PROVIDER: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
}
pub(super) fn card_provides_prevention(card: Card) -> bool {
    card.static_ability_groups().iter().any(|group| {
        group.fragments.iter().any(|fragment| {
            matches!(
                fragment,
                crate::cards::StaticAbilityFragment::PreventControlledPermanentGraveyardMove
            )
        })
    })
}
impl Game {
    pub(super) fn register_stack_graveyard_exile(&mut self, item: &SpellView) {
        if item.kind == StackKind::Spell
            && !item.is_copy
            && item.cast.is_some_and(|cast| cast.exile_on_graveyard)
            && let Some(source) = item.source
        {
            let rows = &mut self
                .state
                .zone_rules
                .get_or_insert_default()
                .graveyard_exile_spells;
            if !rows.contains(&source.object) {
                rows.push(source.object);
            }
        }
    }
    pub(super) fn forget_stack_graveyard_exile(&mut self, object: ObjectRef) {
        if let Some(rules) = &mut self.state.zone_rules
            && rules.graveyard_exile_spells.contains(&object)
        {
            rules
                .graveyard_exile_spells
                .retain(|reference| *reference != object);
        }
    }
    fn stack_graveyard_exile(&self, object: ObjectRef) -> bool {
        self.state
            .zone_rules
            .as_ref()
            .is_some_and(|rules| rules.graveyard_exile_spells.contains(&object))
    }

    pub(super) fn register_zone_rule_card(&mut self, card: Card) {
        if card.zone_replacement().is_some() || card_provides_prevention(card) {
            self.state.zone_rules.get_or_insert_default();
        }
    }
    pub(super) fn record_zone_reveal(&mut self, card: Card, owner: Player, from: Zone) {
        self.state
            .zone_rules
            .get_or_insert_default()
            .reveals
            .push(ZoneRevealView { card, owner, from });
    }
    pub(super) fn clear_zone_reveals(&mut self) {
        if let Some(rules) = &mut self.state.zone_rules {
            rules.reveals.clear();
        }
    }
    pub(super) fn zone_reveals(&self) -> Vec<ZoneRevealView> {
        self.state
            .zone_rules
            .as_ref()
            .map_or_else(Vec::new, |r| r.reveals.to_vec())
    }
    pub(super) fn zone_choice_view(&self, viewer: Player) -> Option<ZoneReplacementChoiceView> {
        let Pending::ZoneReplacement { player } = self.state.pending else {
            return None;
        };
        self.shares_information(viewer, player).then(|| {
            self.state
                .zone_rules
                .as_ref()
                .unwrap()
                .choice
                .as_ref()
                .unwrap()
                .request
                .view
                .clone()
        })
    }
    pub(super) fn zone_choice_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        self.state
            .zone_rules
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .request
            .view
            .choices
            .iter()
            .any(|&effect| emit(Action::ChooseZoneReplacement { effect }))
    }
    pub(super) fn freeze_zone_rules(
        &self,
        before: &ObjectView,
        requested: Zone,
        discard_cause: Option<Player>,
    ) -> FrozenRules {
        let prevent = self.zone_move_prevented(before.reference, requested);
        if requested != Zone::Graveyard {
            return FrozenRules {
                prevent,
                ..FrozenRules::default()
            };
        }
        let own = before.card.zone_replacement();
        let own_available =
            before.zone != Zone::Battlefield || self.printed_abilities_available(before.id);
        let card_or_token =
            !self.state.objects[before.id as usize].is_spell_copy() || before.is_token();
        let spell = before.kinds & 12 != 0 && !before.is_token() && card_or_token;
        let reflexive_exiles = if before.zone == Zone::Battlefield && before.is_creature() {
            self.state
                .static_sources
                .iter()
                .filter_map(|&id| {
                    let source = &self.state.objects[id as usize];
                    let Some(ZoneReplacementRule::ExileOpposingCreatureDeath { ability }) =
                        source.card.zone_replacement()
                    else {
                        return None;
                    };
                    (self.on_battlefield(id)
                        && source.controller != before.controller
                        && self.printed_abilities_available(id))
                    .then(|| (ability, self.object_view(id)))
                })
                .collect()
        } else {
            Vec::new()
        };
        FrozenRules {
            prevent,
            reflexive_exiles,
            shuffle_self: own_available && own == Some(ZoneReplacementRule::RevealShuffleSelf),
            exile_spell: (before.zone == Zone::Stack
                && self.stack_graveyard_exile(before.reference))
                || self.state.static_sources.iter().any(|&id| {
                    self.on_battlefield(id)
                        && self.printed_abilities_available(id)
                        && match self.state.objects[id as usize].card.zone_replacement() {
                            Some(ZoneReplacementRule::ExileAllCardsAndTokens) => card_or_token,
                            Some(ZoneReplacementRule::ExileInstantSorceryCards) => spell,
                            Some(ZoneReplacementRule::ExileOpposingCreatureDeathPlain) => {
                                before.zone == Zone::Battlefield
                                    && before.is_creature()
                                    && self.state.objects[id as usize].controller
                                        != before.controller
                            }
                            _ => false,
                        }
                }),
            discard_to_battlefield: before.zone == Zone::Hand
                && own == Some(ZoneReplacementRule::OpponentDiscardToBattlefield)
                && discard_cause.is_some_and(|cause| cause != before.owner),
        }
    }
    pub(super) fn plan_zone_move(
        &mut self,
        before: &ObjectView,
        requested: Zone,
        rules: FrozenRules,
    ) -> Plan {
        if let Some(plan) = self.replay_frozen_mana_zone_plan(before.reference, requested) {
            return plan;
        }
        let plan = self.plan_zone_move_unrecorded(before, requested, rules);
        self.record_cast_mana_zone_plan(before, &plan);
        plan
    }
    fn plan_zone_move_unrecorded(
        &mut self,
        before: &ObjectView,
        requested: Zone,
        rules: FrozenRules,
    ) -> Plan {
        #[cfg(test)]
        let test_redirect = requested == Zone::Exile
            && TEST_REDIRECT_EXILE_TO_BATTLEFIELD
                .with(|target| target.get() == Some(before.reference));
        #[cfg(not(test))]
        let test_redirect = false;
        if !rules.prevent
            && !test_redirect
            && requested != Zone::Graveyard
            && !(before.zone == Zone::Stack && self.state.objects[before.id as usize].flashback)
        {
            return Plan {
                destination: requested,
                prevented: false,
                shuffle: false,
                reflexive: None,
            };
        }
        let mut choices = Vec::new();
        if rules.prevent {
            choices.push(ZoneReplacementKind::Prevent);
        }
        if test_redirect {
            choices.push(ZoneReplacementKind::Battlefield);
        }
        if before.zone == Zone::Stack && self.state.objects[before.id as usize].flashback
            || rules.exile_spell
            || self.death_exile_destination(before, requested) != requested
        {
            choices.push(ZoneReplacementKind::Exile);
        }
        for (ability, source) in &rules.reflexive_exiles {
            choices.push(ZoneReplacementKind::ExileReflexive {
                source: source.reference,
                ability: *ability,
            });
        }
        if requested == Zone::Graveyard && rules.shuffle_self {
            choices.push(ZoneReplacementKind::RevealShuffle);
        }
        if requested == Zone::Graveyard && rules.discard_to_battlefield {
            choices.push(ZoneReplacementKind::Battlefield);
        }
        let effect = match choices.as_slice() {
            [] => {
                return Plan {
                    destination: requested,
                    prevented: false,
                    shuffle: false,
                    reflexive: None,
                };
            }
            [effect] => *effect,
            _ => {
                if let Some(replay) = self
                    .state
                    .zone_rules
                    .as_mut()
                    .and_then(|r| r.replay.as_mut())
                    && let Some(pick) = replay.base.picks.get(replay.next)
                {
                    assert_eq!(
                        (pick.object, pick.requested),
                        (before.reference, requested),
                        "exact zone replay prefix"
                    );
                    assert!(
                        choices.contains(&pick.effect),
                        "live zone replacement choice"
                    );
                    replay.next += 1;
                    return Self::zone_plan(pick.effect, &rules);
                }
                let request = Request {
                    object: before.reference,
                    requested,
                    affected: if before.zone == Zone::Battlefield {
                        before.controller
                    } else {
                        before.owner
                    },
                    view: ZoneReplacementChoiceView {
                        card: before.card,
                        owner: before.owner,
                        from: before.zone,
                        choices,
                    },
                };
                // Private typed unwind uses the existing action checkpoint and never calls the panic hook.
                std::panic::resume_unwind(Box::new(request));
            }
        };
        Self::zone_plan(effect, &rules)
    }
    fn zone_plan(effect: ZoneReplacementKind, rules: &FrozenRules) -> Plan {
        Plan {
            destination: match effect {
                ZoneReplacementKind::Prevent => Zone::Graveyard,
                ZoneReplacementKind::RevealShuffle => Zone::Library,
                ZoneReplacementKind::Exile | ZoneReplacementKind::ExileReflexive { .. } => {
                    Zone::Exile
                }
                ZoneReplacementKind::Battlefield => Zone::Battlefield,
            },
            prevented: effect == ZoneReplacementKind::Prevent,
            shuffle: effect == ZoneReplacementKind::RevealShuffle,
            reflexive: if let ZoneReplacementKind::ExileReflexive { source, ability } = effect {
                Some((
                    ability,
                    Box::new(
                        rules
                            .reflexive_exiles
                            .iter()
                            .find(|(index, view)| *index == ability && view.reference == source)
                            .expect("frozen selected replacement source")
                            .1
                            .clone(),
                    ),
                ))
            } else {
                None
            },
        }
    }
    pub(super) fn reveal_and_shuffle_zone_card(&mut self, before: &ObjectView) {
        self.state
            .zone_rules
            .get_or_insert_default()
            .reveals
            .push(ZoneRevealView {
                card: before.card,
                owner: before.owner,
                from: before.zone,
            });
        self.forget_library_top(before.owner);
        self.shuffle_library_cards(before.owner);
    }
    pub(super) fn suspend_zone_replacement(
        &mut self,
        request: Request,
        checkpoint: State,
        actor: Player,
        action: Option<Action>,
        revision: u64,
    ) -> Result<(), Error> {
        let Some(next_revision) = revision.checked_add(1) else {
            self.state = checkpoint;
            self.revision = revision;
            return Err(Error::CapacityExceeded(crate::capacity::CapacityExceeded {
                resource: "revision exhausted",
            }));
        };
        let replay = self.state.zone_rules.as_mut().and_then(|r| r.replay.take());
        let base = if let Some(replay) = replay {
            replay.base
        } else {
            let current = std::mem::replace(&mut self.state, checkpoint);
            let bytes = self.state_bytes();
            let state = std::mem::replace(&mut self.state, current);
            assert!(
                state
                    .zone_rules
                    .as_ref()
                    .is_none_or(|r| r.choice.is_none() && r.replay.is_none()),
                "nonrecursive zone checkpoint"
            );
            Base {
                state: Box::new(state),
                bytes,
                actor,
                action: action.expect("zone-aware action"),
                picks: SharedVec::default(),
            }
        };
        let affected = request.affected;
        self.state.zone_rules.get_or_insert_default().choice =
            Some(Box::new(Choice { base, request }));
        self.state.pending = Pending::ZoneReplacement { player: affected };
        self.revision = next_revision;
        Ok(())
    }
    pub(super) fn choose_zone_replacement(&mut self, effect: ZoneReplacementKind) {
        let foreign_cast_choice = self.foreign_cast_choice_pending();
        let choice = self
            .state
            .zone_rules
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let mut base = choice.base;
        base.picks.push(Pick {
            object: choice.request.object,
            requested: choice.request.requested,
            effect,
        });
        let actor = base.actor;
        let action = base.action.clone();
        let revision = self.revision;
        let mana_replay = self.state.mana_lots.as_mut().and_then(|r| r.replay.take());
        self.state = (*base.state).clone();
        if let Some(mana_replay) = mana_replay {
            self.state.mana_lots.get_or_insert_default().replay = Some(mana_replay);
        }
        self.state.zone_rules.get_or_insert_default().replay =
            Some(Box::new(Replay { base, next: 0 }));
        let result = if matches!(self.state.pending, Pending::ManaLotSpend { .. }) {
            self.apply_mana_lot_choice(actor, action)
        } else {
            self.apply_transaction(actor, action)
        };
        result.expect("saved legal zone action remains legal");
        let replay = self
            .state
            .zone_rules
            .as_mut()
            .unwrap()
            .replay
            .take()
            .unwrap();
        assert_eq!(
            replay.next,
            replay.base.picks.len(),
            "complete zone replay prefix"
        );
        self.revision = revision;
        // The saved action runs as its original actor. Retain the actual foreign
        // replacement decision only after replay bookkeeping has been consumed.
        if foreign_cast_choice {
            self.mark_foreign_cast_choice();
        }
    }
}

/// A failed casting proposal cannot conceal a real irreversible library reveal.
/// The choice/replay stack and stack-to-graveyard permissions remain checkpoint-owned.
pub(super) fn preserve_irreversible_reveals(
    target: &mut Option<Box<ZoneState>>,
    source: Option<&ZoneState>,
) {
    if let Some(source) = source {
        if !source.reveals.is_empty() || target.is_some() {
            target.get_or_insert_default().reveals = source.reveals.clone();
        }
    } else if let Some(target) = target {
        target.reveals.clear();
    }
}

/// Preserve only the currently live replay when restoring a staged cost.
/// A checkpoint can outlive the replay that created it, or can fail while
/// that replay is still on the caller's stack.
pub(super) fn transfer_live_replay(
    from: &mut Option<Box<ZoneState>>,
    to: &mut Option<Box<ZoneState>>,
) {
    let live = from.as_mut().and_then(|rules| rules.replay.take());
    if live.is_some() || to.is_some() {
        to.get_or_insert_default().replay = live;
    }
}
