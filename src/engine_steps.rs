//! Shared live predicates, beginning-of-step events and intervening-if checks.
use super::*;
impl Game {
    pub(super) fn begin_step(&mut self, step: Step) {
        self.state.step = step;
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
                        CastRelation::You => source.owner == self.state.active,
                        CastRelation::Opponent => source.owner != self.state.active,
                        CastRelation::Any => true,
                    };
                    if matches_step && related {
                        self.queue_trigger(&source, index);
                    }
                }
            }
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
            TriggerCondition::OncePerTurn(event) | TriggerCondition::DuringOpponentsTurn(event) => {
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
        let p = &self.state.players[player.index()];
        let (value, minimum, maximum) = match guard {
            StepGuard::Always => return true,
            StepGuard::Expression {
                value,
                minimum,
                maximum,
            } => {
                let value = self.stat_value(value, source, player);
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
            o.zone == Zone::Battlefield && o.owner == player
        };
        let creatures = || {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| owned(*id) && o.card.is_creature())
        };
        let value = match value {
            StepValue::Life => Some(p.life),
            StepValue::Hand => Some(self.hand_count(player) as i128),
            StepValue::SpellsCast => Some(i128::from(p.spells_cast_this_turn)),
            StepValue::Creatures => Some(creatures().count() as i128),
            StepValue::TappedCreatures => {
                Some(creatures().filter(|(_, o)| o.tapped).count() as i128)
            }
            StepValue::Lands => Some(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| owned(*id) && o.card.is_land())
                    .count() as i128,
            ),
            StepValue::GreatestCreaturePower => creatures()
                .map(|(id, _)| self.stats(id as CardId).power)
                .max(),
            StepValue::GreatestOtherCreaturePower => creatures()
                .filter(|(id, _)| self.object_ref(*id as CardId) != source)
                .map(|(id, _)| self.stats(id as CardId).power)
                .max(),
            StepValue::TotalCreaturePower => Some(
                creatures()
                    .map(|(id, _)| self.stats(id as CardId).power)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
            StepValue::TotalCreatureToughness => Some(
                creatures()
                    .map(|(id, _)| self.stats(id as CardId).toughness)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
        };
        value.is_some_and(|value| {
            minimum.is_none_or(|min| value >= min) && maximum.is_none_or(|max| value <= max)
        })
    }
}
