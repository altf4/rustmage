//! Mayhem remembers exact discarded graveyard objects, never later reentries.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_mayhem_discards(&mut self, events: &[ZoneChange]) {
        for event in events {
            // Only immutable definitions with Mayhem can use this permission.
            // Keeping their sparse public history avoids allocating for unrelated discards.
            if event.moved
                && event.destination == Zone::Graveyard
                && event.was_card
                && event.before.card.mayhem_cost().is_some()
            {
                let rows = self.state.mayhem_discards.get_or_insert_default();
                let at = rows.partition_point(|row| row.card < event.after.card);
                rows.insert(at, event.after);
            }
        }
    }
    pub(super) fn forget_mayhem_discard(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.mayhem_discards {
            rows.retain(|row| *row != object);
            if rows.is_empty() {
                self.state.mayhem_discards = None;
            }
        }
    }
    pub(super) fn can_cast_mayhem(&self, player: Player, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        object.owner == player
            && object.zone == Zone::Graveyard
            && object.is_card()
            && object.card.mayhem_cost().is_some()
            && self
                .state
                .mayhem_discards
                .as_ref()
                .is_some_and(|rows| rows.contains(&self.object_ref(id)))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReturnedAbilityGrant {
    grantor: ObjectRef,
    definition: Card,
    controller: Player,
    ability: u16,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn return_target_with_attack_trigger(&mut self, item: &SpellView, ability: u16) {
        let Some(source) = item.source else {
            return;
        };
        let grant = ReturnedAbilityGrant {
            grantor: source.object,
            definition: item.trigger_definition(),
            controller: item.controller,
            ability,
        };
        let moves = item
            .targets
            .iter()
            .copied()
            .filter(|&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                movement.controller = item.controller;
                movement.attack_trigger_after = Some(Box::new(grant.clone()));
                Some(movement)
            })
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn grant_returned_attack_trigger(
        &mut self,
        recipient: ObjectRef,
        grant: &ReturnedAbilityGrant,
    ) {
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        assert!(
            grant.definition.triggers()[usize::from(grant.ability)]
                .condition
                .is_granted()
        );
        let timestamp = self.next_role_timestamp();
        self.grant_recipient_ability(
            recipient,
            grant.definition,
            SelfGrantedAbilityKind::AttacksEachCombat,
            timestamp,
            None,
        );
        self.state
            .temporary_abilities
            .get_or_insert_default()
            .push(TemporaryAbilityView {
                recipient,
                grantor: grant.grantor,
                granting_card: grant.definition,
                ability: grant.ability,
                grantor_controller: grant.controller,
                expires_turn: self.state.turn,
                permanent: true,
                timestamp,
                origin: Some(timestamp),
            });
    }
}
