//! Frozen public event facts, recorded even before an observing permanent exists.
use super::*;
use crate::cards::{CastFilter, CreatureCastQualifier, StaticAbilityFragment, TriggeredAbility};
use std::sync::Arc;

/// Cumulative history. An arriving card need not still be in the graveyard.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) struct ZoneChangeTurnHistory {
    pub(super) creature_card_arrivals: [u64; 2],
    pub(super) artifact_permanent_arrivals: u64,
    pub(super) artifact_sacrifices: [bool; 2],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ZoneChangeTurnHistory {
    pub(super) fn record_creature_card_arrival(&mut self, owner: Player) {
        let count = &mut self.creature_card_arrivals[owner.index()];
        *count = count
            .checked_add(1)
            .capacity("creature card arrival history exhausted");
    }

    pub(super) fn record_artifact_cost_departure(
        &mut self,
        controller: Player,
        requested: Zone,
        destination: Zone,
    ) {
        // Battlefield-to-graveyard cost proposals are sacrifices. A different
        // destination can replace the move without changing its cause.
        if requested == Zone::Graveyard && destination != Zone::Battlefield {
            self.artifact_sacrifices[controller.index()] = true;
        }
        if destination == Zone::Graveyard {
            self.record_artifact_permanent_arrival();
        }
    }

    pub(super) fn record_artifact_permanent_arrival(&mut self) {
        self.artifact_permanent_arrivals = self
            .artifact_permanent_arrivals
            .checked_add(1)
            .capacity("artifact permanent arrival history exhausted");
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TurnEventHistory {
    zone_changes: ZoneChangeTurnHistory,
    damage_sources: [Vec<ObjectRef>; 2],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TurnEventHistory {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .damage_sources
                .iter()
                .map(|rows| std::mem::size_of_val(rows.as_slice()))
                .sum::<usize>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn turn_event_history_mut(&mut self) -> &mut TurnEventHistory {
        Arc::make_mut(
            self.state
                .turn_event_history
                .get_or_insert_with(Default::default),
        )
    }

    pub(super) fn zone_change_turn_history(&self) -> ZoneChangeTurnHistory {
        self.state
            .turn_event_history
            .as_ref()
            .map_or(ZoneChangeTurnHistory::default(), |history| {
                history.zone_changes
            })
    }

    pub(super) fn record_artifact_sacrifice(&mut self, payer: Player) {
        if !self.zone_change_turn_history().artifact_sacrifices[payer.index()] {
            self.turn_event_history_mut()
                .zone_changes
                .artifact_sacrifices[payer.index()] = true;
        }
    }

    pub(super) fn record_graveyard_arrival(&mut self, id: CardId, before: &ObjectView) {
        let card = &self.state.objects[id as usize];
        let owner = card.owner;
        let creature_card =
            before.zone != Zone::Graveyard && card.is_card() && self.is_creature(id);
        let artifact_permanent = before.zone == Zone::Battlefield && before.kinds & 16 != 0;
        if creature_card || artifact_permanent {
            let history = &mut self.turn_event_history_mut().zone_changes;
            if creature_card {
                history.record_creature_card_arrival(owner);
            }
            if artifact_permanent {
                history.record_artifact_permanent_arrival();
            }
        }
    }

    pub(super) fn record_damage_source_this_turn(&mut self, source: DamageSource) {
        let player = source.controller.index();
        let position = self
            .state
            .turn_event_history
            .as_ref()
            .map_or(Err(0), |history| {
                history.damage_sources[player].binary_search(&source.reference)
            });
        if let Err(position) = position {
            self.turn_event_history_mut().damage_sources[player].insert(position, source.reference);
        }
    }

    pub(super) fn distinct_damage_sources_this_turn(&self, controller: Player) -> usize {
        self.state.turn_event_history.as_ref().map_or(0, |history| {
            history.damage_sources[controller.index()].len()
        })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TurnFactsView {
    /// A permanent actually left under this player's pre-event control.
    #[serde(skip_serializing_if = "is_false")]
    pub permanent_left_battlefield: bool,
    pub artifact_entered: bool,
    pub hero_entered: bool,
    pub hero_attacked: bool,
    /// Counts begun combat phases, capped at two.
    #[serde(skip_serializing_if = "is_zero")]
    pub combat_phases_started: u8,
    /// Counts begun main phases, capped after the second.
    #[serde(skip_serializing_if = "is_zero")]
    pub main_phases_started: u8,
    #[serde(skip_serializing_if = "is_zero_kinds")]
    pub cast_card_kinds: u16,
    /// Human and legendary creature casts, each capped at two.
    #[serde(skip_serializing_if = "no_qualified_creature_casts")]
    pub qualified_creature_casts: [u8; 2],
    /// Committed casts of flying non-Lemur creature spells, capped at two.
    #[serde(skip_serializing_if = "is_zero")]
    pub flying_non_lemur_creature_casts: u8,
    /// Actual committed activation history, retained even before a provider enters.
    #[serde(skip_serializing_if = "is_false")]
    pub exhaust_activated_this_turn: bool,
    /// A nonempty coin-flip batch has taken place for this player this turn.
    #[serde(skip_serializing_if = "is_false")]
    pub coin_flipped_this_turn: bool,
    /// Actual spells or land plays outside this player's hand.
    #[serde(skip_serializing_if = "is_false")]
    pub played_from_outside_own_hand: bool,
    /// A committed spell, excluding lands and special actions, from this player's hand.
    #[serde(skip_serializing_if = "is_false")]
    pub cast_spell_from_own_hand: bool,
    /// Actual stack mana value, including the chosen face and X.
    #[serde(skip_serializing_if = "is_false")]
    pub cast_mana_value_at_least_four: bool,
    /// A face-down permanent entered under this player's control or they turned one face up.
    #[serde(skip_serializing_if = "is_false")]
    pub face_down_entry_or_turn_up: bool,
    /// An exact face-down object entered as a creature under this player's control.
    #[serde(skip_serializing_if = "is_false")]
    pub face_down_creature_entered: bool,
    /// Bending completed this turn, in air, earth, fire, water bit order.
    #[serde(skip_serializing_if = "is_zero")]
    pub bent_elements: u8,
}

fn no_qualified_creature_casts(counts: &[u8; 2]) -> bool {
    *counts == [0; 2]
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TurnFactsView {
    pub(super) fn qualified_creature_cast_count(self, qualifier: CreatureCastQualifier) -> u8 {
        match qualifier {
            CreatureCastQualifier::FlyingNonLemur => self.flying_non_lemur_creature_casts,
            _ => self.qualified_creature_casts[qualifier.index()],
        }
    }

    pub(super) fn record_qualified_creature_cast(
        &mut self,
        card: Card,
        traits: programs::SpellCastTraits,
    ) {
        for qualifier in CreatureCastQualifier::ALL {
            let count = match qualifier {
                CreatureCastQualifier::FlyingNonLemur => &mut self.flying_non_lemur_creature_casts,
                _ => &mut self.qualified_creature_casts[qualifier.index()],
            };
            if *count < 2 && traits.matches_qualifier(qualifier, card) {
                *count += 1;
            }
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &u8) -> bool {
    *value == 0
}

fn is_zero_kinds(value: &u16) -> bool {
    *value == 0
}

/// Frozen at declaration, before any per-attacker triggered-ability callbacks.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug)]
pub(super) struct AttackPowerEvent {
    pub previous_maximum: Option<i128>,
    pub total: i128,
}

/// Keep the immutable face and the uncommon effective subtype overrides.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CreatureTypeCast {
    object: ObjectRef,
    controller: Player,
    card: Card,
    subtypes: Option<Box<GrantedSubtypes>>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn creature_type_cast_bytes(rows: &Option<SharedVec<CreatureTypeCast>>) -> usize {
    rows.as_ref().map_or(0, |rows| {
        rows.len() * std::mem::size_of::<CreatureTypeCast>()
            + rows.iter().filter(|row| row.subtypes.is_some()).count()
                * std::mem::size_of::<GrantedSubtypes>()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn is_first_other_creature_type_cast(
        &self,
        caster: Player,
        source: ObjectRef,
        subtype: &str,
        spell: Card,
        spell_subtypes: GrantedSubtypes,
    ) -> bool {
        spell_subtypes.has(spell, subtype)
            && !self
                .state
                .creature_type_casts
                .iter()
                .flat_map(|casts| casts.iter())
                .any(|cast| {
                    cast.controller == caster
                    // Spell-to-permanent entry advances this physical object's
                    // generation exactly once. Blink/recast has a new source.
                    && !(cast.object.card == source.card
                        && cast.object.generation.checked_add(1) == Some(source.generation))
                    && cast.subtypes.as_deref().copied().unwrap_or_default().has(cast.card, subtype)
                })
    }

    pub(super) fn record_creature_type_cast(
        &mut self,
        event: &CastPaymentEvent,
        kinds: u16,
        subtypes: GrantedSubtypes,
    ) {
        // Only creatures and kindred spells can have a creature type.
        if kinds & (1 | 128) != 0 {
            self.state
                .creature_type_casts
                .get_or_insert_default()
                .push(CreatureTypeCast {
                    object: event.object,
                    controller: event.controller,
                    card: event.card,
                    subtypes: (subtypes != GrantedSubtypes::default()).then(|| Box::new(subtypes)),
                });
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn guard_uses_facts(guard: StepGuard) -> bool {
    matches!(
        guard,
        StepGuard::PermanentLeftBattlefieldThisTurn
            | StepGuard::ArtifactEnteredThisTurn
            | StepGuard::HeroEnteredOrAttackedThisTurn
            | StepGuard::PlayedFromOutsideOwnHandThisTurn
            | StepGuard::NoSpellCastFromOwnHandThisTurn
            | StepGuard::CastManaValueAtLeastFourThisTurn
            | StepGuard::FirstCombatPhase
            | StepGuard::SecondMainPhase
    )
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn condition_uses_facts(condition: TriggerCondition) -> bool {
    match condition {
        TriggerCondition::AtStep { guard, .. } => guard_uses_facts(guard),
        TriggerCondition::SpellCast {
            filter: CastFilter::FirstQualifiedCreature(_),
            ..
        }
        | TriggerCondition::BendsYou => true,
        TriggerCondition::When { guard, event }
        | TriggerCondition::InterveningIf { guard, event } => {
            guard_uses_facts(guard) || condition_uses_facts(*event)
        }
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::OncePerIncarnation(event)
        | TriggerCondition::DuringOpponentsTurn(event)
        | TriggerCondition::CastFromHand(event)
        | TriggerCondition::CastFromExile(event)
        | TriggerCondition::PlayFromOutsideOwnHand(event)
        | TriggerCondition::TreasureManaSpent(event)
        | TriggerCondition::ManaSpentBelowValue(event)
        | TriggerCondition::ManaSpent { event, .. } => condition_uses_facts(*event),
        _ => false,
    }
}
fn trigger_uses_facts(ability: &TriggeredAbility) -> bool {
    condition_uses_facts(ability.condition)
        || ability.program.iter().any(|i| {
            matches!(
                i,
                Instruction::DrawScaled(_, AmountValue::CastCardTypeCount)
                    | Instruction::SkipIfFaceDownEntryOrTurnUp(_)
            )
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_entry_turn_facts(&mut self, events: &[ZoneChange]) {
        if !events
            .iter()
            .any(|e| e.moved && e.destination == Zone::Battlefield)
        {
            return;
        }
        // Every simultaneous newcomer/provider/attachment is installed already.
        self.refresh_controllers();
        let mut facts = [TurnFactsView::default(); 2];
        let mut entered_types = [subtypes::CreatureTypeSet::default(); 2];
        for event in events {
            if !event.moved
                || event.destination != Zone::Battlefield
                || self.object_ref(event.after.card) != event.after
                || !self.on_battlefield(event.after.card)
            {
                continue;
            }
            let id = event.after.card;
            let object = &self.state.objects[id as usize];
            let controller = object.controller;
            let subtypes = self.granted_subtypes(id);
            let zombie = subtypes.has(object.card, "Zombie");
            facts[controller.index()].hero_entered |= subtypes.has(object.card, "Hero");
            // Registration predates the full simultaneous entry. Freeze the
            // actual controller now that continuous control has settled.
            if let Some(entry) = self
                .state
                .entered_this_turn
                .as_mut()
                .and_then(|rows| rows.iter_mut().find(|entry| entry.object == event.after))
            {
                entry.controller = controller;
                entry.zombie = zombie;
            }
            let kinds = self.current_kinds(id);
            let face_down = self.is_face_down(event.after);
            facts[controller.index()].face_down_entry_or_turn_up |= face_down;
            facts[controller.index()].face_down_creature_entered |= face_down && kinds & 1 != 0;
            if kinds & (1 | 128) != 0 {
                entered_types[controller.index()].union(subtypes.creature_type_set(object.card));
            }
            facts[controller.index()].artifact_entered |= kinds & 16 != 0;
            if kinds & 2 != 0 {
                let count = &mut self.state.players[controller.index()].lands_entered_this_turn;
                *count = count
                    .checked_add(1)
                    .capacity("land entry history exhausted");
            }
        }
        if entered_types.iter().any(|types| !types.is_empty()) {
            let history = std::sync::Arc::make_mut(
                self.state
                    .entered_creature_types
                    .get_or_insert_with(Default::default),
            );
            for (record, new) in history.iter_mut().zip(entered_types) {
                record.union(new);
            }
        }
        for (player, event) in self.state.players.iter_mut().zip(facts) {
            player.turn_facts.artifact_entered |= event.artifact_entered;
            player.turn_facts.hero_entered |= event.hero_entered;
            player.turn_facts.face_down_entry_or_turn_up |= event.face_down_entry_or_turn_up;
            player.turn_facts.face_down_creature_entered |= event.face_down_creature_entered;
        }
    }
    pub(super) fn subtype_entered_under_control(&self, player: Player, types: &[&str]) -> bool {
        self.state
            .entered_creature_types
            .as_ref()
            .is_some_and(|rows| {
                types
                    .iter()
                    .any(|subtype| rows[player.index()].contains(subtype))
            })
    }
    pub(super) fn record_attack_turn_facts(
        &mut self,
        actor: Player,
        declared: &[AttackDeclaration],
    ) -> AttackPowerEvent {
        let total = declared
            .iter()
            .filter(|attack| self.is_creature(attack.card))
            .try_fold(0i128, |total, attack| {
                total.checked_add(self.stats(attack.card).power)
            })
            .capacity("declared attack power representation exhausted");
        let previous_maximum =
            self.state.players[actor.index()].greatest_declared_attack_power_this_turn;
        if !declared.is_empty() {
            self.state.players[actor.index()].greatest_declared_attack_power_this_turn =
                Some(previous_maximum.map_or(total, |prior| prior.max(total)));
        }
        self.record_declared_attack_history(actor, declared);
        // CR508.1f/k/m: inspect characteristics at the actual attack event, after taps.
        let hero = declared.iter().any(|attack| {
            self.on_battlefield(attack.card)
                && self.state.objects[attack.card as usize].controller == actor
                && self.state.attackers.iter().any(|a| a.card == attack.card)
                && self.has_subtype(attack.card, "Hero")
        });
        self.state.players[actor.index()].turn_facts.hero_attacked |= hero;
        AttackPowerEvent {
            previous_maximum,
            total,
        }
    }
    pub(super) fn turn_facts_view(&self) -> Option<[TurnFactsView; 2]> {
        // Public opt-in only. Never scan hidden hands/libraries for observers.
        // Suppression does not hide a publicly known definition; old queued packets
        // keep the facts visible after their source leaves or changes controller.
        let observer = self.state.players.iter().any(|p| {
            p.turn_facts.exhaust_activated_this_turn
                || p.turn_facts.coin_flipped_this_turn
                || p.turn_facts.bent_elements != 0
        }) || self.state.objects.iter().any(|o| {
            o.zone == Zone::Battlefield
                && (o.card.triggers().iter().any(trigger_uses_facts)
                    || o.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| match fragment {
                            StaticAbilityFragment::KeywordGrant(_, grant) => {
                                grant.condition == Some(StatValue::ArtifactEnteredThisTurn)
                            }
                            StaticAbilityFragment::CostReduction(_, rule) => {
                                matches!(rule.filter, CastFilter::FirstQualifiedCreature(_))
                            }
                            _ => false,
                        })
                    }))
        }) || self
            .state
            .stack
            .iter()
            .chain(&self.state.pending_triggers)
            .chain(&self.state.ordering_triggers)
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .chain(self.state.announcing.iter().map(|a| &a.item))
            .any(|item| match item.kind {
                StackKind::TriggeredAbility { ability } => item
                    .trigger_definition()
                    .triggers()
                    .get(usize::from(ability))
                    .is_some_and(trigger_uses_facts),
                _ => false,
            });
        observer.then(|| std::array::from_fn(|i| self.state.players[i].turn_facts))
    }
}
