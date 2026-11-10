//! Frozen public event facts, recorded even before an observing permanent exists.
use super::*;
use crate::cards::{CastFilter, CreatureCastQualifier, StaticAbilityFragment, TriggeredAbility};

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
    #[serde(skip_serializing_if = "is_zero_kinds")]
    pub cast_card_kinds: u16,
    /// Human and legendary creature casts, each capped at two.
    #[serde(skip_serializing_if = "no_qualified_creature_casts")]
    pub qualified_creature_casts: [u8; 2],
    /// Actual committed activation history, retained even before a provider enters.
    #[serde(skip_serializing_if = "is_false")]
    pub exhaust_activated_this_turn: bool,
    /// A nonempty coin-flip batch has taken place for this player this turn.
    #[serde(skip_serializing_if = "is_false")]
    pub coin_flipped_this_turn: bool,
}

fn no_qualified_creature_casts(counts: &[u8; 2]) -> bool {
    *counts == [0; 2]
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TurnFactsView {
    pub(super) fn qualified_creature_cast_count(self, qualifier: CreatureCastQualifier) -> u8 {
        self.qualified_creature_casts[qualifier.index()]
    }

    pub(super) fn record_qualified_creature_cast(&mut self, card: Card) {
        for qualifier in CreatureCastQualifier::ALL {
            let count = &mut self.qualified_creature_casts[qualifier.index()];
            if *count < 2 && qualifier.matches(card) {
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn guard_uses_facts(guard: StepGuard) -> bool {
    matches!(
        guard,
        StepGuard::PermanentLeftBattlefieldThisTurn
            | StepGuard::ArtifactEnteredThisTurn
            | StepGuard::HeroEnteredOrAttackedThisTurn
            | StepGuard::FirstCombatPhase
    )
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn condition_uses_facts(condition: TriggerCondition) -> bool {
    match condition {
        TriggerCondition::AtStep { guard, .. } => guard_uses_facts(guard),
        TriggerCondition::SpellCast {
            filter: CastFilter::FirstQualifiedCreature(_),
            ..
        } => true,
        TriggerCondition::When { guard, event }
        | TriggerCondition::InterveningIf { guard, event } => {
            guard_uses_facts(guard) || condition_uses_facts(*event)
        }
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::DuringOpponentsTurn(event)
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
        for event in events {
            if !event.moved
                || event.destination != Zone::Battlefield
                || self.object_ref(event.after.card) != event.after
                || !self.on_battlefield(event.after.card)
            {
                continue;
            }
            let id = event.after.card;
            let controller = self.state.objects[id as usize].controller;
            let kinds = self.current_kinds(id);
            facts[controller.index()].artifact_entered |= kinds & 16 != 0;
            if kinds & 2 != 0 {
                let count = &mut self.state.players[controller.index()].lands_entered_this_turn;
                *count = count
                    .checked_add(1)
                    .capacity("land entry history exhausted");
            }
            facts[controller.index()].hero_entered |= self.has_subtype(id, "Hero");
        }
        for (player, event) in self.state.players.iter_mut().zip(facts) {
            player.turn_facts.artifact_entered |= event.artifact_entered;
            player.turn_facts.hero_entered |= event.hero_entered;
        }
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
            p.turn_facts.exhaust_activated_this_turn || p.turn_facts.coin_flipped_this_turn
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
