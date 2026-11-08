//! Shared actual-transition evidence for the fast and comprehensive TMT games.
use rustmage::{Action, CastMethod, EffectChoiceKind, Observation, Player, StackKind, Zone};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct Evidence {
    pub(crate) executed: BTreeMap<&'static str, u64>,
    pub(crate) casts: BTreeSet<&'static str>,
    pub(crate) triggers: BTreeSet<&'static str>,
    pub(crate) games: usize,
    pub(crate) decisions: usize,
    pub(crate) branches: usize,
    pub(crate) private: usize,
    pub(crate) replayed: usize,
    pub(crate) combat: usize,
    pub(crate) drawn_to_empty: usize,
    pub(crate) wins: [usize; 2],
    pub(crate) draws: usize,
}
impl Evidence {
    pub(crate) fn add(&mut self, name: &'static str, amount: u64) {
        *self.executed.entry(name).or_default() += amount;
    }
    pub(crate) fn transition(
        &mut self,
        before: &Observation,
        after: &Observation,
        action: &Action,
    ) {
        let before_objects = before
            .objects
            .iter()
            .map(|o| (o.id, o))
            .collect::<BTreeMap<_, _>>();
        if matches!(action, Action::ChooseKicker { pay: true }) {
            self.add("kicker choices", 1);
        }
        if matches!(action, Action::ChooseAdditionalCost { .. }) {
            self.add("additional costs selected", 1);
        }
        if matches!(action, Action::ChooseMode { .. }) {
            self.add("modal choices", 1);
        }
        if matches!(action, Action::SelectTargetCounter { .. }) {
            self.add("counter removals selected", 1);
        }
        if matches!(action, Action::CastExiledDuringResolution { .. }) {
            self.add("resolution cast choices", 1);
        }
        for packet in &after.stack {
            if before.stack.iter().any(|p| p.id == packet.id) {
                continue;
            }
            let Some((_, card)) = packet.source_identity() else {
                continue;
            };
            if packet.kind == StackKind::Spell {
                self.casts.insert(card.name());
                if let Some(cast) = packet.cast {
                    if cast.method == CastMethod::Sneak {
                        self.add("sneak casts committed", 1);
                    }
                    if cast.method == CastMethod::Free {
                        self.add("free casts committed", 1);
                    }
                    if cast.kicker == Some(true) {
                        self.add("kicked casts committed", 1);
                    }
                    if cast.method == CastMethod::Normal
                        && card.name() == "Krang, Master Mind"
                        && cast.payment.into_iter().map(u64::from).sum::<u64>()
                            < card.fixed_mana_value()
                    {
                        self.add("artifact affinity discounted casts", 1);
                    }
                    if cast.origin == Zone::Exile {
                        self.add("exile casts committed", 1);
                    }
                }
            } else {
                self.triggers.insert(card.name());
            }
        }
        // ability_resolutions is a rules history used only by abilities that
        // explicitly count their own resolutions, not a general event log.
        // Observe a packet leaving the stack/resolution across this accepted
        // action instead. A consumed packet can fizzle; the mechanic evidence
        // below additionally requires its actual visible counter/damage effect.
        let finishing = before.resolving.as_ref().or_else(|| {
            matches!(action, Action::Pass)
                .then(|| before.stack.last())
                .flatten()
        });
        if let Some(packet) = finishing
            && !after.stack.iter().any(|p| p.id == packet.id)
            && after.resolving.as_ref().is_none_or(|p| p.id != packet.id)
        {
            if matches!(packet.kind, StackKind::TriggeredAbility { .. }) {
                self.add("triggered ability packets consumed", 1);
                if let Some((source, card)) = packet.source_identity()
                    && card.name() == "Michelangelo, Game Master"
                    && matches!(packet.kind, StackKind::TriggeredAbility { ability: 0 })
                    && after.objects.iter().any(|o| {
                        o.reference == source
                            && o.zone == Zone::Battlefield
                            && before.objects.iter().any(|old| {
                                old.reference == o.reference
                                    && old.zone == Zone::Battlefield
                                    && o.counters.plus_one > old.counters.plus_one
                            })
                    })
                {
                    self.add("disappear counter abilities resolved", 1);
                }
                if packet
                    .source_identity()
                    .is_some_and(|(_, card)| card.name() == "Raphael, Tough Turtle")
                    && matches!(packet.kind, StackKind::TriggeredAbility { ability: 0 })
                    && after.players[packet.controller.other().index()].life
                        < before.players[packet.controller.other().index()].life
                {
                    self.add("alliance damage abilities resolved", 1);
                }
                if packet.source_identity().is_some_and(|(_, card)| {
                    matches!(
                        card.name(),
                        "Casey Jones, Vigilante" | "Koya, Death from Above"
                    )
                }) && matches!(packet.kind, StackKind::TriggeredAbility { ability: 1 })
                {
                    self.add("delayed step packets consumed", 1);
                }
            }
            if matches!(packet.kind, StackKind::ActivatedAbility { .. }) {
                self.add("activated ability packets consumed", 1);
            }
        }
        for row in after.delayed_steps.iter().flatten() {
            if !before
                .delayed_steps
                .iter()
                .flatten()
                .any(|old| old.item.id == row.item.id)
            {
                self.add("delayed step programs scheduled", 1);
            }
        }
        let outside_choice = before.outside_choice.is_some()
            || before.outside_entry_replacement.is_some()
            || before
                .legendary_split_search
                .as_ref()
                .is_some_and(|search| search.revealed);
        for object in &after.objects {
            let old = before_objects
                .get(&object.id)
                .copied()
                .filter(|o| o.reference == object.reference);
            if let Some(level) = object.class_level {
                let previous = old.and_then(|o| o.class_level).unwrap_or(1);
                if level > previous {
                    self.add("class levels gained", u64::from(level - previous));
                }
            }
            if outside_choice
                && object.zone == Zone::Hand
                && object.reference.generation == 0
                && old.is_none()
            {
                self.add("outside cards moved to hand", 1);
            }
            if object.zone == Zone::Battlefield && old.is_none_or(|o| o.zone != Zone::Battlefield) {
                if object.is_token() {
                    self.add("tokens entered", 1);
                }
                if before_objects
                    .get(&object.id)
                    .is_some_and(|o| o.zone == Zone::Graveyard)
                {
                    self.add("graveyard returns", 1);
                }
            }
            if object.zone == Zone::Battlefield
                && object.copy_recipe.is_some()
                && old.is_none_or(|o| {
                    o.copy_recipe != object.copy_recipe || o.zone != Zone::Battlefield
                })
            {
                self.add("copy effects applied", 1);
            }
            if object.counters.get(rustmage::CounterKind::Hexproof)
                > old.map_or(0, |o| o.counters.get(rustmage::CounterKind::Hexproof))
            {
                self.add("hexproof counters placed", 1);
            }
            if object.counters.plus_one > old.map_or(0, |o| o.counters.plus_one) {
                self.add("plus counters placed", 1);
            }
        }
        if !after.attackers.is_empty() {
            self.combat += 1;
        }
    }
}

fn private_effect_choice(kind: EffectChoiceKind) -> bool {
    matches!(
        kind,
        EffectChoiceKind::Discard
            | EffectChoiceKind::HandToTop
            | EffectChoiceKind::BottomHand
            | EffectChoiceKind::HandCreature
            | EffectChoiceKind::HandCreatureAndLand
            | EffectChoiceKind::HandPermanent { .. }
            | EffectChoiceKind::HandLand { .. }
    )
}

pub(crate) fn private_choice(observation: &Observation) -> bool {
    observation
        .legendary_split_search
        .as_ref()
        .is_some_and(|c| !c.revealed)
        || observation
            .outside_choice
            .as_ref()
            .is_some_and(|c| c.pool.is_some() && c.revealed.is_none())
        || observation.library_choice.is_some()
        || observation.library_selection.is_some()
        || observation.search_choice.is_some()
        || observation.connive_choice.is_some()
        || observation.hand_choice.is_some()
        || observation
            .entry_value
            .as_ref()
            .is_some_and(|v| v.opponent_hand.is_some())
        || observation
            .effect_choice
            .as_ref()
            .is_some_and(|c| private_effect_choice(c.kind))
}

pub(crate) fn check_privacy(view: &Observation, actor: Player) {
    assert!(view.objects.iter().all(|o| o.zone != Zone::Hand
        || o.owner == view.viewer
        || view.known_hand.contains(&o.reference)));
    assert!(view.objects.iter().all(|o| o.zone != Zone::Outside));
    assert!(
        view.owned_outside
            .as_ref()
            .is_none_or(|pools| pools[view.viewer.other().index()].is_none())
    );
    if let Some(choice) = &view.outside_choice
        && choice.owner != view.viewer
    {
        assert!(choice.pool.is_none() && choice.first.is_none());
    }
    if view.viewer != actor {
        assert!(view.actions.is_empty());
        assert!(view.library_choice.is_none());
        assert!(
            view.library_selection
                .as_ref()
                .is_none_or(|selection| selection.public)
        );
        assert!(view.search_choice.is_none());
        assert!(view.connive_choice.is_none());
        assert!(view.hand_choice.is_none());
        assert!(
            view.entry_value
                .as_ref()
                .is_none_or(|value| value.opponent_hand.is_none())
        );
        assert!(
            view.effect_choice
                .as_ref()
                .is_none_or(|choice| !private_effect_choice(choice.kind))
        );
        if let Some(search) = &view.legendary_split_search {
            assert!(search.revealed);
            assert!(
                search.library.is_empty()
                    && search.library_candidates.is_empty()
                    && search.outside_candidates.is_empty()
            );
        }
    }
}
