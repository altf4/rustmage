//! Delayed clauses keep their original controller and source independently of
//! the watched creature's control, abilities, or exact source's later departure.
use super::*;
use crate::cards::DelayedTargetEvent;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedTargetTriggerView {
    pub subject: ObjectRef,
    pub event: DelayedTargetEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_turn: Option<u32>,
    pub packet: SpellView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn delay_target_event(
        &mut self,
        item: &SpellView,
        event: DelayedTargetEvent,
        ability: u16,
    ) {
        Self::check_delayed_target_template(item, event, ability);
        let mut subjects = Vec::new();
        for &bound in &item.targets {
            if let Target::Card(id) = bound.target
                && self.same_target_object(bound)
                && self.on_battlefield(id)
            {
                let subject = self.object_ref(id);
                if !subjects.contains(&subject) {
                    subjects.push(subject);
                }
            }
        }
        for subject in subjects {
            let mut packet = self.delayed_target_packet(item, ability);
            packet.refer_objects([subject]);
            self.state
                .delayed_target_events
                .get_or_insert_default()
                .push(DelayedTargetTriggerView {
                    subject,
                    event,
                    expires_turn: (event != DelayedTargetEvent::LeavesBattlefieldOnce)
                        .then_some(self.state.turn),
                    packet,
                });
        }
    }
    fn check_delayed_target_template(item: &SpellView, event: DelayedTargetEvent, ability: u16) {
        let definition = item.trigger_definition();
        let trigger = &definition.triggers()[usize::from(ability)];
        let condition = match event {
            DelayedTargetEvent::ControlLostWhileAttached => {
                TriggerCondition::DelayedTargetControlLostWhileAttached
            }
            _ => TriggerCondition::DelayedTargetEvent,
        };
        assert_eq!(trigger.condition, condition);
        assert!(
            !trigger.targets.has_targets(),
            "delayed target template must be untargeted"
        );
    }

    fn delayed_target_packet(&mut self, item: &SpellView, ability: u16) -> SpellView {
        let mut packet = item.clone();
        packet.id = self.next_stack_id();
        packet.kind = StackKind::TriggeredAbility { ability };
        packet.targets.clear();
        packet.clear_initiating_event_references();
        packet.target_type_slots = None;
        packet.mode = None;
        packet.cast = None;
        packet.related_stack = None;
        packet.is_copy = false;
        packet.additional_triggers = 0;
        packet
    }

    pub(super) fn register_created_token_departure(
        &mut self,
        item: &SpellView,
        ability: u16,
        created: Vec<ObjectRef>,
    ) {
        let Some(source) = item.source else { return };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        let event = DelayedTargetEvent::LeavesBattlefieldOnce;
        Self::check_delayed_target_template(item, event, ability);
        let mut packet = self.delayed_target_packet(item, ability);
        // Keep the exact created objects distinct from the later source move.
        packet.set_reflexive_action_objects(created);
        self.state
            .delayed_target_events
            .get_or_insert_default()
            .push(DelayedTargetTriggerView {
                subject: source.object,
                event,
                expires_turn: None,
                packet,
            });
    }

    pub(super) fn exile_delayed_created_tokens(&mut self, item: &SpellView) {
        let ids: Vec<_> = item
            .reflexive_action_objects()
            .filter(|object| {
                self.on_battlefield(object.card) && self.object_ref(object.card) == *object
            })
            .map(|object| object.card)
            .collect();
        if !ids.is_empty() {
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.finish_zone_events(&events, &[]);
        }
    }

    pub(super) fn expire_turn_delayed_target_events(&mut self) {
        if let Some(rows) = &mut self.state.delayed_target_events {
            rows.retain(|row| row.expires_turn.is_none());
            if rows.is_empty() {
                self.state.delayed_target_events = None;
            }
        }
    }

    pub(super) fn delayed_target_departure(&mut self, event: &ZoneChange) {
        if !event.moved || event.before.zone != Zone::Battlefield {
            return;
        }
        if !self
            .state
            .delayed_target_events
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.subject == event.before.reference))
        {
            return;
        }
        let Some(rows) = self.state.delayed_target_events.take() else {
            return;
        };
        let dies = event.destination == Zone::Graveyard;
        let mut retained = Vec::new();
        for row in *rows {
            if row.expires_turn.is_some_and(|turn| turn != self.state.turn) {
                continue;
            }
            if row.subject != event.before.reference {
                retained.push(row);
                continue;
            }
            let matches = match row.event {
                DelayedTargetEvent::LeavesBattlefieldOnce => true,
                DelayedTargetEvent::DiesOnce => dies,
                DelayedTargetEvent::GraveyardUnderControllerOnce => {
                    event.destination == Zone::Graveyard
                        && event.before.controller == row.packet.controller
                }
                DelayedTargetEvent::CombatDamageToPlayer
                | DelayedTargetEvent::CombatDamage
                | DelayedTargetEvent::ControlLostWhileAttached => false,
            };
            if matches {
                let mut packet = row.packet;
                packet.id = self.next_stack_id();
                packet.refer_zone_event(event);
                let program = packet.trigger_definition().triggers()[match packet.kind {
                    StackKind::TriggeredAbility { ability } => usize::from(ability),
                    _ => unreachable!(),
                }]
                .program;
                self.state.pending_triggers.push(packet);
                self.capture_battlefield_departure_facts(event, program);
            } else {
                // CR603.7b/c: now-inert exact subjects are still referred to by
                // this turn-duration delayed ability; a new incarnation cannot
                // match it, and the turn advance ends its lifetime.
                retained.push(row);
            }
        }
        if !retained.is_empty() {
            self.state.delayed_target_events = Some(Box::new(retained));
        }
    }
    pub(super) fn delayed_target_control_losses(
        &mut self,
        losses: &[(ObjectRef, Player)],
        attached_before: &[(ObjectRef, Player)],
    ) {
        if attached_before.is_empty() {
            return;
        }
        let packets: Vec<_> = self
            .state
            .delayed_target_events
            .iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| {
                let loss = (row.subject, row.packet.controller);
                row.event == DelayedTargetEvent::ControlLostWhileAttached
                    && row.expires_turn == Some(self.state.turn)
                    && losses.contains(&loss)
                    && attached_before.contains(&loss)
            })
            .map(|row| row.packet.clone())
            .collect();
        for mut packet in packets {
            packet.id = self.next_stack_id();
            self.state.pending_triggers.push(packet);
        }
    }
    pub(super) fn delayed_target_combat_damage(&mut self, events: &[DamageEvent]) {
        let Some(rows) = self.state.delayed_target_events.as_ref() else {
            return;
        };
        let watch_any = rows
            .iter()
            .any(|row| row.event == DelayedTargetEvent::CombatDamage);
        let mut player_hits = std::collections::BTreeSet::new();
        let mut amounts = std::collections::BTreeMap::<ObjectRef, u128>::new();
        for event in events
            .iter()
            .filter(|event| event.combat && event.amount > 0)
        {
            let dealer = event.source.reference;
            if let Target::Player(player) = event.target {
                player_hits.insert((dealer, player));
            }
            if watch_any {
                let amount = amounts.entry(dealer).or_default();
                *amount = amount
                    .checked_add(event.amount as u128)
                    .capacity("combat damage amount exhausted");
            }
        }
        let mut packets = Vec::new();
        for row in rows
            .iter()
            .filter(|row| row.expires_turn == Some(self.state.turn))
        {
            match row.event {
                DelayedTargetEvent::CombatDamageToPlayer => {
                    packets.extend(
                        player_hits
                            .iter()
                            .filter(|(dealer, _)| *dealer == row.subject)
                            .map(|_| row.packet.clone()),
                    );
                }
                DelayedTargetEvent::CombatDamage => {
                    if let Some(&amount) = amounts.get(&row.subject) {
                        let mut packet = row.packet.clone();
                        packet.creature_combat_event = Some(Box::new(CreatureCombatEventView {
                            dealer: row.subject,
                            amount,
                        }));
                        packets.push(packet);
                    }
                }
                _ => (),
            }
        }
        for mut packet in packets {
            packet.id = self.next_stack_id();
            self.state.pending_triggers.push(packet);
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[allow(
    clippy::box_collection,
    reason = "The optional cold collection keeps one pointer in State"
)]
pub(super) fn owned_bytes(rows: &Option<Box<Vec<DelayedTargetTriggerView>>>) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<DelayedTargetTriggerView>>()
            + rows
                .iter()
                .map(|row| {
                    std::mem::size_of::<DelayedTargetTriggerView>() + row.packet.owned_bytes()
                })
                .sum::<usize>()
    })
}
