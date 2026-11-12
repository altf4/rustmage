//! Owned turn-duration delayed triggers; sources and recipients never rebind.
use super::*;
use crate::cards::TurnCallbackEvent;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TurnCallbackView {
    pub event: TurnCallbackEvent,
    pub once: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<TurnCallbackExpiry>,
    pub item: SpellView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum TurnCallbackExpiry {
    /// The next actual turn of this packet's controller has not yet begun.
    EndOfNextTurn,
    /// Expire in this specific turn's cleanup.
    EndOfTurn(u32),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn is_player_or_planeswalker_damage(&self, event: &DamageEvent) -> bool {
        match event.target {
            Target::Player(_) => true,
            Target::Card(id) => self.on_battlefield(id) && self.current_kinds(id) & 64 != 0,
            _ => false,
        }
    }
    pub(super) fn creature_death_turn_callbacks(&mut self, events: &[ZoneChange]) {
        if !self.state.combat_rules.as_ref().is_some_and(|rules| {
            rules.turn_callbacks.iter().any(|row| {
                matches!(
                    row.event,
                    TurnCallbackEvent::AnyCreatureDeath
                        | TurnCallbackEvent::ControlledCreatureDeath
                        | TurnCallbackEvent::ControlledNontokenCreatureDeath
                )
            })
        }) {
            return;
        }
        for event in events {
            if event.moved
                && event.before.zone == Zone::Battlefield
                && event.destination == Zone::Graveyard
                && event.before.kinds & 1 != 0
            {
                self.queue_matching_turn_callbacks(None, |row| {
                    let controlled = row.item.controller == event.before.controller;
                    match row.event {
                        TurnCallbackEvent::AnyCreatureDeath => true,
                        TurnCallbackEvent::ControlledCreatureDeath => controlled,
                        TurnCallbackEvent::ControlledNontokenCreatureDeath => {
                            controlled && !event.before.is_token()
                        }
                        _ => false,
                    }
                });
            }
        }
    }

    pub(super) fn combat_turn_callback_events(&mut self, events: &[DamageEvent]) {
        let Some(rules) = &self.state.combat_rules else {
            return;
        };
        let mut controlled = false;
        let mut named = false;
        let mut player_targeted = false;
        for row in &rules.turn_callbacks {
            match row.event {
                TurnCallbackEvent::ControlledCreatureCombatDamagePlayerOrPlaneswalker => {
                    controlled = true;
                }
                TurnCallbackEvent::NamedCreatureCombatDamagePlayer(_) => named = true,
                TurnCallbackEvent::ControlledCreatureCombatDamagePlayer(_) => {
                    player_targeted = true;
                }
                _ => {}
            }
        }
        if !controlled && !named && !player_targeted {
            return;
        }
        let hits: std::collections::BTreeSet<_> = events
            .iter()
            .filter(|event| {
                event.amount > 0
                    && event.combat
                    && event.source.facts.battlefield
                    && event.source.facts.is_creature()
                    && self.is_player_or_planeswalker_damage(event)
            })
            .map(|event| {
                (
                    event.source.controller,
                    event.source.reference,
                    event.target,
                )
            })
            .collect();
        for (controller, object, recipient) in hits {
            let name = (named
                && matches!(recipient, Target::Player(_))
                && self.on_battlefield(object.card)
                && self.object_ref(object.card) == object)
                .then(|| self.object_name(object.card));
            self.queue_matching_turn_callbacks(Some(object), |row| match row.event {
                TurnCallbackEvent::ControlledCreatureCombatDamagePlayerOrPlaneswalker => {
                    row.item.controller == controller
                }
                TurnCallbackEvent::NamedCreatureCombatDamagePlayer(chosen) => {
                    name.is_some_and(|name| chosen.matches(name))
                }
                TurnCallbackEvent::ControlledCreatureCombatDamagePlayer(player) => {
                    row.item.controller == controller && recipient == Target::Player(player)
                }
                _ => false,
            });
        }
    }
    pub(super) fn schedule_turn_callback(
        &mut self,
        item: &SpellView,
        event: TurnCallbackEvent,
        ability: u16,
        once: bool,
        expiry: Option<TurnCallbackExpiry>,
    ) {
        let mut packet = item.clone();
        packet.id = self.next_stack_id();
        packet.kind = StackKind::TriggeredAbility { ability };
        packet.targets.clear();
        packet.clear_initiating_event_references();
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
                expiry,
                item: packet,
            });
    }
    pub(super) fn expire_next_turn_callbacks(&mut self) {
        if let Some(rules) = &mut self.state.combat_rules {
            let expiry = Some(TurnCallbackExpiry::EndOfTurn(self.state.turn));
            rules
                .turn_callbacks
                .retain(|row| row.expiry.is_some() && row.expiry != expiry);
        }
    }

    pub(super) fn queue_cast_turn_callbacks(
        &mut self,
        caster: Player,
        object: Option<ObjectRef>,
        noncreature: bool,
    ) {
        self.queue_matching_turn_callbacks(object, |row| {
            row.item.controller == caster
                && match row.event {
                    TurnCallbackEvent::SpellCast => object.is_some(),
                    TurnCallbackEvent::NoncreatureCast => noncreature,
                    _ => false,
                }
        });
    }

    pub(super) fn queue_turn_callbacks(
        &mut self,
        event: TurnCallbackEvent,
        actor: Player,
        object: Option<ObjectRef>,
    ) {
        self.queue_matching_turn_callbacks(object, |row| {
            row.event == event && row.item.controller == actor
        });
    }
    fn queue_matching_turn_callbacks(
        &mut self,
        object: Option<ObjectRef>,
        mut matches: impl FnMut(&TurnCallbackView) -> bool,
    ) {
        let Some(rules) = self.state.combat_rules.as_mut() else {
            return;
        };
        let mut ready = Vec::new();
        rules.turn_callbacks.retain(|row| {
            if !matches(row) {
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
    pub(super) fn allow_attack_without_defender(&mut self, object: ObjectRef) {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return;
        }
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        rules
            .attack_without_defender
            .retain(|r| r.card != object.card || *r == object);
        if !rules.attack_without_defender.contains(&object) {
            rules.attack_without_defender.push(object);
        }
    }
}
