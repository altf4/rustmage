//! Exact linked, turn-limited free casting with an explicitly selected approving ability.
use super::*;
use crate::cards::StaticAbilityFragment;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LinkedFreeCastUseView {
    pub source: ObjectRef,
    pub ability_origin: u64,
    pub player: Player,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LinkedFreeCastApprovalView {
    pub source: ObjectRef,
    pub ability_origin: u64,
    pub player: Player,
    pub stack_id: u64,
}
impl Game {
    pub(super) fn exile_target_library_linked(&mut self, item: &SpellView, amount: u16) {
        for player in self.program_players(PlayerSet::Targeted, item) {
            let ids: Vec<_> = self.state.players[player.index()]
                .library
                .iter()
                .rev()
                .take(usize::from(amount))
                .copied()
                .collect();
            let events = self.move_objects_raw(&ids, Zone::Exile);
            let actual: Vec<_> = events
                .iter()
                .filter(|e| e.moved && e.destination == Zone::Exile)
                .map(|e| e.after.card)
                .collect();
            self.remember_linked_exile_for_origin(item.object, item.ability_origin(), &actual);
            self.finish_zone_events(&events, &[]);
        }
    }
    fn linked_free_cast_limit(
        &self,
        player: Player,
        source: ObjectRef,
        card: CardId,
    ) -> Option<u128> {
        if self.object_ref(source.card) != source
            || !self.on_battlefield(source.card)
            || self.state.objects[source.card as usize].controller != player
            || !self.printed_abilities_available(source.card)
        {
            return None;
        }
        let data = self.state.linked_choices.as_ref()?;
        if data.free_cast_uses.iter().any(|row| {
            row.source == source
                && row.player == player
                && row.ability_origin == self.copy_ability_origin(source)
        }) || !data.links.iter().any(|row| {
            row.source == source
                && row.ability_origin == self.copy_ability_origin(source)
                && row.exiled == self.object_ref(card)
                && row.exiled_turn == self.state.turn
        }) {
            return None;
        }
        let definition = self.state.objects[source.card as usize].card;
        let subtypes = definition
            .static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .find_map(|f| match f {
                StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(types) => Some(*types),
                _ => None,
            })?;
        Some(
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == player
                        && subtypes.iter().any(|t| self.has_subtype(*id as CardId, t))
                })
                .count() as u128,
        )
    }
    pub(super) fn has_linked_free_cast(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.static_sources.iter().any(|&id| {
                self.linked_free_cast_limit(player, self.object_ref(id), card)
                    .is_some()
            })
    }
    pub(super) fn visit_linked_free_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for &id in &self.state.static_sources {
            let source = self.object_ref(id);
            let Some(limit) = self.linked_free_cast_limit(player, source, card) else {
                continue;
            };
            let physical = self.state.objects[card as usize].card;
            for face in [CastFace::Normal, CastFace::Adventure, CastFace::Omen] {
                let Some(chosen) = (match face {
                    CastFace::Normal => Some(physical),
                    CastFace::Adventure => physical.adventure_spell(),
                    CastFace::Omen => physical.omen_spell(),
                }) else {
                    continue;
                };
                if u128::from(chosen.fixed_mana_value()) > limit {
                    continue;
                }
                for flash_surcharge in [false, true] {
                    if self.permitted_cast_available(
                        player,
                        card,
                        CastMethod::Free,
                        face,
                        sorcery,
                        flash_surcharge,
                    ) && emit(Action::AnnounceLinkedFree {
                        card,
                        source,
                        face,
                        flash_surcharge,
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub(super) fn commit_linked_free_cast(&mut self, stack_id: u64) {
        let Some(data) = self.state.linked_choices.as_mut() else {
            return;
        };
        if data
            .free_cast_approval
            .is_some_and(|row| row.stack_id == stack_id)
        {
            let row = data.free_cast_approval.take().unwrap();
            data.free_cast_uses.push(LinkedFreeCastUseView {
                source: row.source,
                ability_origin: row.ability_origin,
                player: row.player,
            });
        }
    }
}
