//! Owned casting permission when more than one rule allows the same library spell.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LibraryEntryCounter {
    pub spell: u64,
    pub counter_faces: u8,
    pub ordinary_cast: bool,
    pub selected: Option<bool>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn ordinary_library_creature_permission(&self, player: Player) -> bool {
        cast_origins::has_fragment(self, player, |fragment| {
            matches!(fragment, StaticAbilityFragment::CastCreatureFromLibraryTop)
        })
    }

    pub(super) fn library_counter_choice_available(&self, player: Player, card: CardId) -> bool {
        let definition = self.state.objects[card as usize].card;
        definition.is_creature()
            && self.library_subtype_cast_permission(player, card, definition)
            && self.ordinary_library_creature_permission(player)
    }

    pub(super) fn library_entry_counter(
        &self,
        player: Player,
        card: CardId,
    ) -> Option<LibraryEntryCounter> {
        let object = &self.state.objects[card as usize];
        if object.zone != Zone::Library || self.state.resolving.is_some() {
            return None;
        }
        let definition = object.card;
        let faces = [
            Some(definition),
            definition.adventure_spell(),
            definition.omen_spell(),
            definition.back_face(),
        ];
        let counter_faces = faces
            .into_iter()
            .enumerate()
            .fold(0, |mask, (index, face)| {
                if face.is_some_and(|face| {
                    face.is_creature() && self.library_subtype_cast_permission(player, card, face)
                }) {
                    mask | (1 << index)
                } else {
                    mask
                }
            });
        (counter_faces != 0).then(|| LibraryEntryCounter {
            spell: 0,
            counter_faces,
            ordinary_cast: self.ordinary_library_creature_permission(player),
            selected: None,
        })
    }

    pub(super) fn library_entry_counter_choice(&self, item: &SpellView) -> Option<bool> {
        let cast = item.cast?;
        if cast.method == CastMethod::Disguise {
            return None;
        }
        let counter = self.state.linked_choices.as_ref()?.library_entry_counter?;
        if counter.spell != item.id || counter.counter_faces & (1 << cast.face as u8) == 0 {
            return None;
        }
        if counter.ordinary_cast {
            counter.selected
        } else {
            Some(true)
        }
    }

    pub(super) fn needs_library_entry_counter_choice(&self, item: &SpellView) -> bool {
        item.cast.is_some_and(|cast| {
            cast.method != CastMethod::Disguise
                && self
                    .state
                    .linked_choices
                    .as_ref()
                    .and_then(|state| state.library_entry_counter)
                    .is_some_and(|counter| {
                        counter.spell == item.id
                            && counter.ordinary_cast
                            && counter.selected.is_none()
                            && counter.counter_faces & (1 << cast.face as u8) != 0
                    })
        })
    }

    pub(super) fn select_library_entry_counter(&mut self, with_counter: bool) {
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .library_entry_counter
            .as_mut()
            .unwrap()
            .selected = Some(with_counter);
        self.continue_announcement_choices(false);
    }

    pub(super) fn commit_library_entry_counter(&mut self, item: &SpellView) {
        let choice = self.library_entry_counter_choice(item);
        if let Some(state) = &mut self.state.linked_choices
            && state
                .library_entry_counter
                .is_some_and(|counter| counter.spell == item.id)
        {
            state.library_entry_counter = None;
        }
        if choice == Some(true)
            && let Some(source) = item.source
        {
            self.record_cast_entry_counters(source.object, CounterKind::PlusOne, 1);
        }
    }
}
