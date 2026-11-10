//! Independent one-shot step triggers retain the creating controller and exact links.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedStep {
    pub steps: &'static [Step],
    pub active_player: Option<Player>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_turn: Option<u32>,
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
        objects: impl IntoIterator<Item = ObjectRef>,
        expires_turn: Option<u32>,
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
        packet.refer_objects(objects);
        packet.mode = None;
        packet.target_type_slots = None;
        packet.additional_triggers = 0;
        packet.cast = None;
        packet.is_copy = false;
        self.state
            .linked_choices
            .get_or_insert_default()
            .delayed_steps
            .push(DelayedStep {
                steps,
                active_player,
                expires_turn,
                item: packet,
            });
    }

    pub(super) fn queue_delayed_steps(&mut self, step: Step) {
        let Some(state) = self.state.linked_choices.as_mut() else {
            return;
        };
        if state.delayed_steps.is_empty() {
            return;
        }
        let rows = std::mem::take(&mut state.delayed_steps);
        let (ready, retained): (Vec<_>, Vec<_>) = rows.into_iter().partition(|row| {
            row.steps.contains(&step) && row.active_player.is_none_or(|p| p == self.state.active)
        });
        state.delayed_steps = retained;
        self.state
            .pending_triggers
            .extend(ready.into_iter().map(|row| row.item));
    }

    fn delayed_battlefield_objects(&self, item: &SpellView) -> Vec<CardId> {
        let Some(group) = &item.event_object else {
            return Vec::new();
        };
        let additional = group
            .semantic_objects
            .iter()
            .flat_map(|rows| rows.iter())
            .filter_map(|reference| match *reference {
                DamageSourceRef::Object(object) => Some(object),
                DamageSourceRef::StackObject { .. } => None,
            });
        std::iter::once(group.object)
            .chain(additional)
            .filter(|&object| {
                self.on_battlefield(object.card) && self.object_ref(object.card) == object
            })
            .map(|object| object.card)
            .collect()
    }

    pub(super) fn remove_delayed_object_counters(
        &mut self,
        item: &SpellView,
        kind: CounterKind,
        amount: u64,
    ) {
        for id in self.delayed_battlefield_objects(item) {
            let removed = self.counter_count(id, kind).min(amount);
            self.pay_counters(id, &Counters::of(kind, removed));
        }
    }

    pub(super) fn return_delayed_battlefield_objects_to_hand(&mut self, item: &SpellView) {
        let ids = self.delayed_battlefield_objects(item);
        let events = self.move_objects_raw(&ids, Zone::Hand);
        self.finish_zone_events(&events, &[]);
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
