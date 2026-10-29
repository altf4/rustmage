//! Exact exile permission with a mandatory life alternative to the mana cost.
use super::*;
impl Game {
    /// Select this permission's cost and modifiers, independently of other exile
    /// permissions. In particular another route's "cast this way" replacement
    /// must not attach to a spell cast through the life alternative.
    pub(super) fn configure_exile_life_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().unwrap();
        cast.method = CastMethod::ExileLife;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("cast spell has source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_exile_life_permission(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|row| {
                row.life_instead_of_mana
                    && row.player == player
                    && row.object == self.object_ref(card)
                    && row.expiry.active(self.state.turn)
            })
    }
    pub(super) fn visit_exile_life_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if object.is_token() || !self.has_exile_life_permission(player, card) {
            return false;
        }
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let definition = match face {
                CastFace::Normal => Some(object.card),
                CastFace::Adventure => object.card.adventure_spell(),
                CastFace::Omen => object.card.omen_spell(),
                CastFace::Back => object.card.back_face(),
            };
            if !definition.is_some_and(|d| !d.is_land()) {
                continue;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_cast_available(
                    player,
                    card,
                    CastMethod::ExileLife,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceExileLife {
                    card,
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn exile_top_play_life(
        &mut self,
        library_owner: Player,
        player: Player,
        amount: u128,
    ) {
        let ids = exile::top_library_cards(self, library_owner, amount);
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                let at = self
                    .state
                    .exile_permissions
                    .partition_point(|p| p.object.card < event.after.card);
                self.state.exile_permissions.insert(
                    at,
                    ExilePlayPermission {
                        object: event.after,
                        player,
                        expiry: ExilePlayExpiry::Turn(self.state.turn),
                        spend_any_mana: false,
                        cast_only: false,
                        exile_on_graveyard: false,
                        life_instead_of_mana: true,
                        without_paying_mana_cost: false,
                        requires_subtype: None,
                    },
                );
            }
        }
        self.finish_zone_events(&events, &[]);
    }
}
