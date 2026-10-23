//! Independent, one-shot beginning-of-step programs retain their creating packet.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedStepView {
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub ability: u16,
    pub steps: &'static [Step],
    pub active_player: Option<Player>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedStep {
    pub view: DelayedStepView,
    pub item: SpellView,
}
impl Game {
    pub(super) fn schedule_delayed_step(
        &mut self,
        item: &SpellView,
        step: TriggerStep,
        relation: CastRelation,
        ability: u16,
    ) {
        assert_eq!(
            item.trigger_definition().triggers()[usize::from(ability)].condition,
            TriggerCondition::Reflexive
        );
        let steps: &'static [Step] = match step {
            TriggerStep::Upkeep => &[Step::Upkeep],
            TriggerStep::Draw => &[Step::Draw],
            TriggerStep::Main1 => &[Step::Main1],
            TriggerStep::Main2 => &[Step::Main2],
            TriggerStep::MainPhases => &[Step::Main1, Step::Main2],
            TriggerStep::BeginCombat => &[Step::BeginCombat],
            TriggerStep::EndCombat => &[Step::EndCombat],
            TriggerStep::End => &[Step::End],
        };
        let active_player = match relation {
            CastRelation::You => Some(item.controller),
            CastRelation::Opponent => Some(item.controller.other()),
            CastRelation::Any => None,
        };
        let mut captured = item.clone();
        captured.targets.clear();
        captured.mode = None;
        let view = DelayedStepView {
            id: self.next_stack_id(),
            source: item.object,
            source_card: item.trigger_definition(),
            controller: item.controller,
            ability,
            steps,
            active_player,
        };
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .delayed_steps
            .push(DelayedStep {
                view,
                item: captured,
            });
    }
    pub(super) fn queue_delayed_steps(&mut self, step: Step) {
        let active = self.state.active;
        let mut ready = Vec::new();
        if let Some(roles) = &mut self.state.roles {
            let mut index = 0;
            while index < roles.delayed_steps.len() {
                let row = &roles.delayed_steps[index];
                if row.view.steps.contains(&step)
                    && row.view.active_player.is_none_or(|p| p == active)
                {
                    ready.push(roles.delayed_steps.remove(index));
                } else {
                    index += 1;
                }
            }
        }
        for row in ready {
            let mut item = row.item;
            item.id = self.next_stack_id();
            item.kind = StackKind::TriggeredAbility {
                ability: row.view.ability,
            };
            // This independent delay is not another occurrence of the creating permanent's ability.
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn visible_delayed_steps(&self) -> Vec<DelayedStepView> {
        self.state.roles.as_ref().map_or_else(Vec::new, |r| {
            r.delayed_steps.iter().map(|r| r.view.clone()).collect()
        })
    }
}
