//! Frozen public event facts, recorded even before an observing permanent exists.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TurnFactsView {
    pub artifact_entered: bool,
    pub hero_entered: bool,
    pub hero_attacked: bool,
    /// Actual committed activation history, retained even before a provider enters.
    #[serde(skip_serializing_if = "is_false")]
    pub exhaust_activated_this_turn: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
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
        StepGuard::ArtifactEnteredThisTurn | StepGuard::HeroEnteredOrAttackedThisTurn
    )
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn condition_uses_facts(condition: TriggerCondition) -> bool {
    match condition {
        TriggerCondition::AtStep { guard, .. } => guard_uses_facts(guard),
        TriggerCondition::When { guard, event }
        | TriggerCondition::InterveningIf { guard, event } => {
            guard_uses_facts(guard) || condition_uses_facts(*event)
        }
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::DuringOpponentsTurn(event)
        | TriggerCondition::TreasureManaSpent(event)
        | TriggerCondition::ManaSpent { event, .. } => condition_uses_facts(*event),
        _ => false,
    }
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
            facts[controller.index()].artifact_entered |= self.current_kinds(id) & 16 != 0;
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
        let observer = self
            .state
            .players
            .iter()
            .any(|p| p.turn_facts.exhaust_activated_this_turn)
            || self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && o.card
                        .triggers()
                        .iter()
                        .any(|a| condition_uses_facts(a.condition))
            })
            || self
                .state
                .stack
                .iter()
                .chain(&self.state.pending_triggers)
                .chain(&self.state.ordering_triggers)
                .chain(self.state.resolving.iter().map(|r| &r.item))
                .chain(self.state.announcing.iter().map(|a| &a.item))
                .any(|item| match item.kind {
                    StackKind::TriggeredAbility { ability } => condition_uses_facts(
                        item.trigger_definition().triggers()[usize::from(ability)].condition,
                    ),
                    _ => false,
                });
        observer.then(|| std::array::from_fn(|i| self.state.players[i].turn_facts))
    }
}
