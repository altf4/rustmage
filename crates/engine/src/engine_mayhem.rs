//! Mayhem remembers exact discarded graveyard objects, never later reentries.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CastMethod {
    pub(super) fn is_mayhem(self) -> bool {
        matches!(self, Self::Mayhem | Self::GrantedMayhem)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_mayhem_discards(&mut self, events: &[ZoneChange]) {
        for event in events {
            // A later static grant may give this exact graveyard object Mayhem.
            // Keep public receipts even if the card didn't have it when discarded.
            if event.moved && event.destination == Zone::Graveyard && event.was_card {
                let rows = self.state.mayhem_discards.get_or_insert_default();
                let at = rows.partition_point(|row| row.card < event.after.card);
                rows.insert(at, event.after);
            }
        }
    }

    pub(super) fn forget_mayhem_discard(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.mayhem_discards
            && let Ok(index) = rows.binary_search_by_key(&object.card, |row| row.card)
            && rows[index] == object
        {
            rows.remove(index);
            if rows.is_empty() {
                self.state.mayhem_discards = None;
            }
        }
    }

    pub(super) fn discarded_to_graveyard_this_turn(&self, id: CardId) -> bool {
        self.state.objects[id as usize].zone == Zone::Graveyard
            && self.state.mayhem_discards.as_ref().is_some_and(|rows| {
                rows.binary_search_by_key(&id, |row| row.card)
                    .is_ok_and(|index| rows[index] == self.object_ref(id))
            })
    }

    pub(super) fn visit_mayhem_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        mut emit: impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.discarded_to_graveyard_this_turn(card) {
            return false;
        }
        let granted = self.grants_mana_cost_mayhem(player);
        [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ]
        .into_iter()
        .any(|face| self.visit_mayhem_face(player, card, face, sorcery, granted, true, &mut emit))
    }

    pub(super) fn visit_resolution_mayhem_face(
        &self,
        player: Player,
        card: CardId,
        face: CastFace,
        emit: impl FnMut(Action) -> bool,
    ) -> bool {
        self.discarded_to_graveyard_this_turn(card)
            && self.visit_mayhem_face(
                player,
                card,
                face,
                true,
                self.grants_mana_cost_mayhem(player),
                false,
                emit,
            )
    }

    fn grants_mana_cost_mayhem(&self, player: Player) -> bool {
        cast_origins::has_fragment(self, player, |fragment| {
            matches!(fragment, StaticAbilityFragment::GrantManaCostMayhem)
        })
    }

    #[allow(clippy::too_many_arguments)] // Live grant, chosen face and cast timing are independent.
    fn visit_mayhem_face(
        &self,
        player: Player,
        card: CardId,
        face: CastFace,
        sorcery: bool,
        granted: bool,
        offer_flash_surcharge: bool,
        mut emit: impl FnMut(Action) -> bool,
    ) -> bool {
        if face == CastFace::Normal
            && self.permitted_cast_available(player, card, CastMethod::Mayhem, face, sorcery, false)
            && emit(Action::AnnounceMayhem { card })
        {
            return true;
        }
        granted
            && [false, true].into_iter().any(|flash_surcharge| {
                (!flash_surcharge || offer_flash_surcharge)
                    && self.permitted_cast_available(
                        player,
                        card,
                        CastMethod::GrantedMayhem,
                        face,
                        sorcery,
                        flash_surcharge,
                    )
                    && emit(Action::AnnounceGrantedMayhem {
                        card,
                        face,
                        flash_surcharge,
                    })
            })
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
