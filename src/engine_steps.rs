//! Beginning-of-step events and CR603.4 intervening-if checks.
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
                    guard,
                } = ability.condition
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
                    if matches_step
                        && related
                        && self.step_guard(guard, source.owner, source.reference, source.tapped)
                    {
                        self.queue_trigger(&source, index);
                    }
                }
            }
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
            StepGuard::Attacked => return p.attacked_this_turn,
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
            StepValue::Hand => Some(self.hand_count(player) as i64),
            StepValue::SpellsCast => {
                Some(i64::try_from(p.spells_cast_this_turn).unwrap_or(i64::MAX))
            }
            StepValue::Creatures => Some(creatures().count() as i64),
            StepValue::TappedCreatures => {
                Some(creatures().filter(|(_, o)| o.tapped).count() as i64)
            }
            StepValue::Lands => Some(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| owned(*id) && o.card.is_land())
                    .count() as i64,
            ),
            StepValue::GreatestCreaturePower => creatures()
                .map(|(id, _)| i64::from(self.stats(id as CardId).power))
                .max(),
            StepValue::TotalCreaturePower => Some(
                creatures()
                    .map(|(id, _)| i64::from(self.stats(id as CardId).power))
                    .sum::<i64>(),
            ),
            StepValue::TotalCreatureToughness => Some(
                creatures()
                    .map(|(id, _)| i64::from(self.stats(id as CardId).toughness))
                    .sum::<i64>(),
            ),
        };
        value.is_some_and(|value| {
            minimum.is_none_or(|min| value >= min) && maximum.is_none_or(|max| value <= max)
        })
    }
}
