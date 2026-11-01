//! Source-independent rules created for the captured player until cleanup.
use super::*;
use crate::cards::StaticAbilityFragment;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PlayerCastGrantsView {
    pub artifact_generic_reduction: u64,
    pub next_red_or_green_creature_free: u64,
    pub next_spell_uncounterable: bool,
}
impl Game {
    pub(super) fn player_cast_grants_view(&self, index: usize) -> Option<PlayerCastGrantsView> {
        self.state
            .turn_cast_grants
            .as_ref()
            .map(|rows| rows[index])
            .filter(|row| *row != PlayerCastGrantsView::default())
    }
    pub(super) fn grant_next_spell_uncounterable(&mut self, player: Player) {
        self.state
            .turn_cast_grants
            .get_or_insert_with(Default::default)[player.index()]
        .next_spell_uncounterable = true;
    }
    pub(super) fn consume_next_spell_uncounterable(
        &mut self,
        player: Player,
        stack: Option<usize>,
    ) {
        let Some(index) = stack else {
            return;
        };
        let Some(rows) = self.state.turn_cast_grants.as_mut() else {
            return;
        };
        if !rows[player.index()].next_spell_uncounterable {
            return;
        }
        rows[player.index()].next_spell_uncounterable = false;
        let id = self.state.stack[index].id;
        self.prune_uncounterable_spell_ids();
        self.mark_spell_uncounterable(id);
    }
    pub(super) fn mark_spell_uncounterable(&mut self, id: u64) {
        let ids = self.state.uncounterable_spell_ids.get_or_insert_default();
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    pub(super) fn prune_uncounterable_spell_ids(&mut self) {
        if let Some(ids) = &mut self.state.uncounterable_spell_ids {
            ids.retain(|id| self.state.stack.iter().any(|item| item.id == *id));
            if ids.is_empty() {
                self.state.uncounterable_spell_ids = None;
            }
        }
    }
    pub(super) fn uncounterable_spell_ids_view(&self) -> Vec<u64> {
        self.state
            .uncounterable_spell_ids
            .as_deref()
            .into_iter()
            .flatten()
            .copied()
            .filter(|id| self.state.stack.iter().any(|item| item.id == *id))
            .collect()
    }
    pub(super) fn grant_artifact_spell_reduction(&mut self, player: Player, amount: u32) {
        if amount == 0 {
            return;
        }
        let rows = self
            .state
            .turn_cast_grants
            .get_or_insert_with(Default::default);
        rows[player.index()].artifact_generic_reduction = rows[player.index()]
            .artifact_generic_reduction
            .checked_add(u64::from(amount))
            .capacity("turn spell reduction exhausted");
    }
    pub(super) fn grant_next_creature_free(&mut self, player: Player) {
        let rows = self
            .state
            .turn_cast_grants
            .get_or_insert_with(Default::default);
        rows[player.index()].next_red_or_green_creature_free = rows[player.index()]
            .next_red_or_green_creature_free
            .checked_add(1)
            .capacity("turn free-cast grants exhausted");
    }
    pub(super) fn turn_artifact_spell_reduction(&self, player: Player, card: Card) -> u64 {
        if card.kind_mask() & 16 == 0 {
            return 0;
        }
        self.player_cast_grants_view(player.index())
            .map_or(0, |row| row.artifact_generic_reduction)
    }
    fn turn_free_creature_matches(card: Card) -> bool {
        card.is_creature() && card.colors() & ((1 << 3) | (1 << 4)) != 0
    }
    pub(super) fn turn_free_creature_available(&self, player: Player, card: Card) -> bool {
        Self::turn_free_creature_matches(card)
            && self
                .player_cast_grants_view(player.index())
                .is_some_and(|row| row.next_red_or_green_creature_free > 0)
    }
    pub(super) fn consume_turn_cast_grants(&mut self, player: Player, card: Card) {
        if !Self::turn_free_creature_matches(card) {
            return;
        }
        if let Some(rows) = self.state.turn_cast_grants.as_mut() {
            rows[player.index()].next_red_or_green_creature_free = 0;
            if rows
                .iter()
                .all(|row| *row == PlayerCastGrantsView::default())
            {
                self.state.turn_cast_grants = None;
            }
        }
    }
    pub(super) fn turn_free_library_origin(&self, player: Player, card: CardId) -> bool {
        if self
            .player_cast_grants_view(player.index())
            .is_none_or(|row| row.next_red_or_green_creature_free == 0)
        {
            return false;
        }
        let object = &self.state.objects[card as usize];
        [
            Some(object.card),
            object.card.adventure_spell(),
            object.card.omen_spell(),
            object.card.back_face(),
        ]
        .into_iter()
        .flatten()
        .any(|definition| {
            self.turn_free_creature_available(player, definition)
                && self.can_cast_library_face(player, card, definition)
        })
    }
    pub(super) fn visit_turn_free_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if self
            .player_cast_grants_view(player.index())
            .is_none_or(|row| row.next_red_or_green_creature_free == 0)
        {
            return false;
        }
        let object = &self.state.objects[card as usize];
        if object.is_token() {
            return false;
        }
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let Some(definition) = (match face {
                CastFace::Normal => Some(object.card),
                CastFace::Adventure => object.card.adventure_spell(),
                CastFace::Omen => object.card.omen_spell(),
                CastFace::Back => object.card.back_face(),
            }) else {
                continue;
            };
            if !self.turn_free_creature_available(player, definition) {
                continue;
            }
            let ordinary_origin = match object.zone {
                Zone::Hand => object.owner == player && object.is_card(),
                Zone::Exile => self.can_play_exiled(player, card),
                Zone::Library => self.can_cast_library_face(player, card, definition),
                Zone::Graveyard => self.has_graveyard_cast(player, card, GraveyardCastKind::Cast),
                _ => false,
            };
            let counter_cost =
                (object.zone == Zone::Graveyard && object.owner == player && object.is_card())
                    .then(|| definition.graveyard_counter_cost())
                    .flatten();
            for flash_surcharge in [false, true] {
                let ordinary_available = self.permitted_cast_available(
                    player,
                    card,
                    CastMethod::Free,
                    face,
                    sorcery,
                    flash_surcharge,
                );
                // The legacy hand loop emits the identical selected-face/surcharge
                // alternative with the same availability check, but only for a
                // nonland physical front. Do not discard a distinct back-face route.
                let legacy_hand_alternative = object.zone == Zone::Hand
                    && !object.card.is_land()
                    && self.free_hand_cast_available(player);
                if ordinary_origin
                    && ordinary_available
                    && !legacy_hand_alternative
                    && emit(Action::AnnounceFree {
                        card,
                        face,
                        flash_surcharge,
                    })
                {
                    return true;
                }
                if let Some(amount) = counter_cost
                    && self.permitted_cast_with_origin_cost(
                        player,
                        card,
                        CastMethod::Free,
                        face,
                        sorcery,
                        flash_surcharge,
                        Some(amount),
                    )
                    && emit(Action::AnnounceFreeCounterGraveyard {
                        card,
                        face,
                        flash_surcharge,
                    })
                {
                    return true;
                }
                if player != self.state.active
                    || object.zone != Zone::Graveyard
                    || object.owner != player
                    || !ordinary_available
                {
                    continue;
                }
                for &id in &self.state.static_sources {
                    let provider = &self.state.objects[id as usize];
                    if provider.zone != Zone::Battlefield
                        || provider.controller != player
                        || !self.printed_abilities_available(id)
                        || !provider.card.static_ability_groups().iter().any(|group| {
                            group.fragments.iter().any(|f| {
                                matches!(f, StaticAbilityFragment::GraveyardPermanentTypes)
                            })
                        })
                    {
                        continue;
                    }
                    // The permanent type belongs to the selected spell face, and the
                    // allowance belongs to this exact permission source incarnation.
                    let permission = self.object_ref(id);
                    for permanent_type in PermanentType::ALL {
                        if permanent_type == PermanentType::Land
                            || definition.kind_mask() & permanent_type.mask() == 0
                            || self.state.graveyard_type_uses.as_ref().is_some_and(|rows| {
                                rows.iter().any(|row| {
                                    row.source == permission
                                        && row.turn == self.state.turn
                                        && row.permanent_type == permanent_type
                                })
                            })
                        {
                            continue;
                        }
                        if emit(Action::AnnounceFreeGraveyardPermanent {
                            card,
                            face,
                            flash_surcharge,
                            permission,
                            permanent_type,
                        }) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
