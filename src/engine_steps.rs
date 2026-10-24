//! Shared live predicates, beginning-of-step events and intervening-if checks.
use super::*;
impl Game {
    pub(super) fn begin_step(&mut self, step: Step) {
        self.state.step = step;
        if step == Step::Main1 {
            self.queue_paradigm_triggers();
            self.precombat_lore_counters();
        }
        if step == Step::End {
            self.queue_copy_end_step_abilities();
            self.queue_delayed_sacrifices();
            if let Some(items) = self.state.delayed_artifact_copies.take() {
                self.state.pending_triggers.extend(*items);
            }
            self.queue_delayed_returns();
            self.queue_warp_exiles();
        }
        let sources: Vec<_> = self
            .state
            .step_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::AtStep {
                    step: wanted,
                    relation,
                    ..
                } = ability.condition.event()
                {
                    let matches_step = match wanted {
                        TriggerStep::Upkeep => step == Step::Upkeep,
                        TriggerStep::Draw => step == Step::Draw,
                        TriggerStep::Main1 => step == Step::Main1,
                        TriggerStep::Main2 => step == Step::Main2,
                        TriggerStep::MainPhases => matches!(step, Step::Main1 | Step::Main2),
                        TriggerStep::BeginCombat => step == Step::BeginCombat,
                        TriggerStep::EndCombat => step == Step::EndCombat,
                        TriggerStep::End => step == Step::End,
                    };
                    let related = match relation {
                        CastRelation::You => source.controller == self.state.active,
                        CastRelation::Opponent => source.controller != self.state.active,
                        CastRelation::Any => true,
                    };
                    if matches_step && related {
                        self.queue_trigger(&source, index);
                    }
                }
            }
        }
    }

    pub(super) fn trigger_time_guards_hold(
        &self,
        condition: TriggerCondition,
        player: Player,
        source: ObjectRef,
        tapped: bool,
    ) -> bool {
        match condition {
            TriggerCondition::When { guard, event } => {
                self.step_guard(guard, player, source, tapped)
                    && self.trigger_time_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::InZone { event, .. }
            | TriggerCondition::Granted(event)
            | TriggerCondition::OncePerTurn(event)
            | TriggerCondition::DuringOpponentsTurn(event)
            | TriggerCondition::ManaSpent { event, .. }
            | TriggerCondition::InterveningIf { event, .. } => {
                self.trigger_time_guards_hold(*event, player, source, tapped)
            }
            _ => true,
        }
    }
    pub(super) fn trigger_guards_hold(
        &self,
        condition: TriggerCondition,
        player: Player,
        source: ObjectRef,
        tapped: bool,
    ) -> bool {
        match condition {
            TriggerCondition::InZone { event, .. }
            | TriggerCondition::When { event, .. }
            | TriggerCondition::Granted(event)
            | TriggerCondition::OncePerTurn(event)
            | TriggerCondition::DuringOpponentsTurn(event)
            | TriggerCondition::ManaSpent { event, .. } => {
                self.trigger_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::InterveningIf { guard, event } => {
                self.step_guard(guard, player, source, tapped)
                    && self.trigger_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::AtStep { guard, .. } => {
                self.step_guard(guard, player, source, tapped)
            }
            _ => true,
        }
    }

    pub(super) fn step_guard(
        &self,
        guard: StepGuard,
        player: Player,
        source: ObjectRef,
        source_tapped: bool,
    ) -> bool {
        self.step_guard_without_hand(guard, player, source, source_tapped, None)
    }
    pub(super) fn step_guard_without_hand(
        &self,
        guard: StepGuard,
        player: Player,
        source: ObjectRef,
        source_tapped: bool,
        departing: Option<CardId>,
    ) -> bool {
        let p = &self.state.players[player.index()];
        let (value, minimum, maximum) = match guard {
            StepGuard::Always => return true,
            StepGuard::CardExiledThisTurn => return self.state.cards_exiled_this_turn,
            StepGuard::OpponentControlsMoreCreatures => {
                let count = |controller| {
                    self.state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == controller
                                && self.is_creature(*id as CardId)
                        })
                        .count()
                };
                return count(player.other()) > count(player);
            }
            StepGuard::OpponentHasMoreCardsInHand => {
                let count = |owner| {
                    self.hand_count(owner)
                        - usize::from(departing.is_some_and(|id| {
                            self.state
                                .objects
                                .get(id as usize)
                                .is_some_and(|o| o.zone == Zone::Hand && o.owner == owner)
                        }))
                };
                return count(player.other()) > count(player);
            }
            StepGuard::SourceReceivedCounterThisTurn => {
                return self.put_counter_on_source_this_turn(source, player);
            }
            StepGuard::YourTurn => return self.state.active == player,
            StepGuard::SourcePowerAtLeast(minimum) => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && self.stats_without_hand(source.card, departing).power
                        >= i128::from(minimum);
            }
            StepGuard::SourceToughnessAtMost(maximum) => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && self.stats_without_hand(source.card, departing).toughness
                        <= i128::from(maximum);
            }
            StepGuard::Outlaw => {
                return self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.controller == player
                        && o.zone == Zone::Battlefield
                        && ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                            .iter()
                            .any(|s| self.has_subtype(id as CardId, s))
                });
            }
            StepGuard::Expression {
                value,
                minimum,
                maximum,
            } => {
                let value = self.stat_value_without_hand(value, source, player, departing);
                return minimum.is_none_or(|min| value >= min)
                    && maximum.is_none_or(|max| value <= max);
            }
            StepGuard::GainedLife => return p.life_gained_this_turn > 0,
            StepGuard::Attacked => return p.attacked_this_turn,
            StepGuard::CreatureDied => return self.state.creatures_died_this_turn > 0,
            StepGuard::OpponentLostLife => {
                return self.state.players[player.other().index()].life_lost_this_turn > 0;
            }
            StepGuard::SourceTapped => {
                return if self.on_battlefield(source.card) && self.object_ref(source.card) == source
                {
                    self.state.objects[source.card as usize].tapped
                } else {
                    source_tapped
                };
            }
            StepGuard::Threshold {
                value,
                minimum,
                maximum,
            } => (value, minimum, maximum),
        };
        let owned = |id: usize| {
            let o = &self.state.objects[id];
            o.zone == Zone::Battlefield && o.controller == player
        };
        let creatures = || {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, _)| owned(*id) && self.is_creature(*id as CardId))
        };
        let value = match value {
            StepValue::Life => Some(p.life),
            StepValue::Hand => Some(
                self.hand_count(player) as i128
                    - i128::from(departing.is_some_and(|id| {
                        self.state
                            .objects
                            .get(id as usize)
                            .is_some_and(|o| o.zone == Zone::Hand && o.owner == player)
                    })),
            ),
            StepValue::SpellsCast => Some(i128::from(p.spells_cast_this_turn)),
            StepValue::Creatures => Some(creatures().count() as i128),
            StepValue::EnchantedCreatures => Some(
                creatures()
                    .filter(|(id, _)| self.is_enchanted(*id as CardId))
                    .count() as i128,
            ),
            StepValue::TappedCreatures => {
                Some(creatures().filter(|(_, o)| o.tapped).count() as i128)
            }
            StepValue::Lands => Some(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| owned(*id) && self.current_kinds(*id as CardId) & 2 != 0)
                    .count() as i128,
            ),
            StepValue::GreatestCreaturePower => creatures()
                .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                .max(),
            StepValue::GreatestOtherCreaturePower => creatures()
                .filter(|(id, _)| self.object_ref(*id as CardId) != source)
                .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                .max(),
            StepValue::TotalCreaturePower => Some(
                creatures()
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
            StepValue::TotalCreatureToughness => Some(
                creatures()
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).toughness)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
        };
        value.is_some_and(|value| {
            minimum.is_none_or(|min| value >= min) && maximum.is_none_or(|max| value <= max)
        })
    }
}
