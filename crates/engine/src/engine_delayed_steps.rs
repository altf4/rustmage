//! Independent one-shot step triggers retain the creating controller and exact links.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedStep {
    pub steps: &'static [Step],
    pub active_player: Option<Player>,
    pub item: SpellView,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn schedule_delayed_step(
        &mut self,
        item: &SpellView,
        step: TriggerStep,
        relation: CastRelation,
        ability: u16,
    ) {
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
        let mut packet = item.clone();
        packet.id = self.next_stack_id();
        packet.kind = StackKind::TriggeredAbility { ability };
        packet.targets.clear();
        packet.clear_initiating_event_references();
        packet.mode = None;
        packet.target_type_slots = None;
        packet.additional_triggers = 0;
        packet.cast = None;
        packet.is_copy = false;
        self.state
            .delayed_steps
            .get_or_insert_with(Default::default)
            .push(DelayedStep {
                steps,
                active_player,
                item: packet,
            });
    }

    pub(super) fn queue_delayed_steps(&mut self, step: Step) {
        let Some(rows) = self.state.delayed_steps.take() else {
            return;
        };
        let (ready, retained): (Vec<_>, Vec<_>) = rows.into_iter().partition(|row| {
            row.steps.contains(&step) && row.active_player.is_none_or(|p| p == self.state.active)
        });
        if !retained.is_empty() {
            self.state.delayed_steps = Some(Box::new(retained));
        }
        self.state
            .pending_triggers
            .extend(ready.into_iter().map(|row| row.item));
    }

    pub(super) fn delayed_steps_bytes(&self) -> usize {
        self.state.delayed_steps.as_ref().map_or(0, |rows| {
            std::mem::size_of::<Vec<DelayedStep>>()
                + rows.len() * std::mem::size_of::<DelayedStep>()
                + rows.iter().map(|row| row.item.owned_bytes()).sum::<usize>()
        })
    }

    pub(super) fn capture_delayed_return_target(&mut self, item: &SpellView) {
        let Some(id) = item.targets.iter().find_map(|bound| match bound.target {
            Target::Card(id) if self.same_target_object(*bound) && self.on_battlefield(id) => {
                Some(id)
            }
            _ => None,
        }) else {
            return;
        };
        let events = self.move_objects_raw(&[id], Zone::Exile);
        let returning = events.iter().find_map(|event| {
            (event.moved
                && event.was_card
                && matches!(
                    event.destination,
                    Zone::Exile | Zone::Graveyard | Zone::Command
                ))
            .then(|| {
                Box::new(ReturnLinkView {
                    object: event.after,
                    zone: event.destination,
                    controller: event.before.owner,
                    counter: None,
                    tapped: false,
                    exile_after: false,
                    additional_returning: None,
                })
            })
        });
        self.state.resolving.as_mut().unwrap().item.return_link = returning;
        self.finish_zone_events(&events, &[]);
    }
}
