//! Owned turn-duration delayed triggers; sources and recipients never rebind.
use super::*;
use crate::cards::TurnCallbackEvent;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TurnCallbackView {
    pub event: TurnCallbackEvent,
    pub once: bool,
    pub item: SpellView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn schedule_turn_callback(
        &mut self,
        item: &SpellView,
        event: TurnCallbackEvent,
        ability: u16,
        once: bool,
    ) {
        let mut packet = item.clone();
        packet.id = self.next_stack_id();
        packet.kind = StackKind::TriggeredAbility { ability };
        packet.targets.clear();
        packet.clear_initiating_event_references();
        packet.crime_qualifying_target = false;
        packet.target_type_slots = None;
        packet.mode = None;
        packet.cast = None;
        packet.is_copy = false;
        packet.related_stack = None;
        packet.reflexive_subject = None;
        packet.entering_object = None;
        packet.additional_triggers = 0;
        self.state
            .combat_rules
            .get_or_insert_with(Default::default)
            .turn_callbacks
            .push(TurnCallbackView {
                event,
                once,
                item: packet,
            });
    }
    pub(super) fn queue_turn_callbacks(
        &mut self,
        event: TurnCallbackEvent,
        actor: Player,
        object: Option<ObjectRef>,
    ) {
        let Some(rules) = self.state.combat_rules.as_mut() else {
            return;
        };
        let mut ready = Vec::new();
        rules.turn_callbacks.retain(|row| {
            if row.event != event || row.item.controller != actor {
                return true;
            }
            ready.push(row.item.clone());
            !row.once
        });
        for mut item in ready {
            item.id = self.next_stack_id();
            item.entering_object = object;
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn capture_turn_callback_source_departure(&mut self, before: &ObjectView) {
        if let Some(rules) = &mut self.state.combat_rules {
            for row in &mut rules.turn_callbacks {
                let item = &mut row.item;
                if item
                    .source
                    .is_some_and(|source| source.object == before.reference)
                {
                    item.source.as_mut().unwrap().facts = DamageFacts::from_view(before);
                    item.source_keywords = before.keywords;
                    item.source_tapped = before.tapped;
                    item.source_attachment = before.attached_to;
                    item.refresh_counter_quantities(|kind| before.counters.get(kind));
                }
            }
        }
    }
    pub(super) fn allow_source_attack_without_defender(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        rules
            .attack_without_defender
            .retain(|r| r.card != source.id || *r == source.object);
        if !rules.attack_without_defender.contains(&source.object) {
            rules.attack_without_defender.push(source.object);
        }
    }
}
