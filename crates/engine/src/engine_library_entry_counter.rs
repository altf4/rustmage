//! Owned casting permission when more than one rule allows the same library spell.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LibraryEntryCounter {
    pub spell: u64,
    pub counter_faces: u8,
    pub ordinary_faces: u8,
    pub selected: Option<bool>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn library_counter_choice_available(&self, player: Player, card: CardId) -> bool {
        let definition = self.state.objects[card as usize].card;
        definition.is_creature()
            && self.top_library_card(player) == Some(card)
            && cast_origins::library_spell_with_counter_permission(self, player, definition)
            && cast_origins::ordinary_library_spell_permission(self, player, definition)
    }

    pub(super) fn library_entry_counter(
        &self,
        player: Player,
        card: CardId,
    ) -> Option<LibraryEntryCounter> {
        let object = &self.state.objects[card as usize];
        if object.zone != Zone::Library
            || self.top_library_card(player) != Some(card)
            || self.state.resolving.is_some()
        {
            return None;
        }
        let definition = object.card;
        let faces = [
            Some(definition),
            definition.adventure_spell(),
            definition.omen_spell(),
            definition.back_face(),
        ];
        let mut counter_faces = 0;
        let mut ordinary_faces = 0;
        for (index, face) in faces.into_iter().enumerate() {
            if let Some(face) = face
                && face.is_creature()
                && cast_origins::library_spell_with_counter_permission(self, player, face)
            {
                counter_faces |= 1 << index;
                if cast_origins::ordinary_library_spell_permission(self, player, face) {
                    ordinary_faces |= 1 << index;
                }
            }
        }
        (counter_faces != 0).then_some(LibraryEntryCounter {
            spell: 0,
            counter_faces,
            ordinary_faces,
            selected: None,
        })
    }

    pub(super) fn library_entry_counter_choice(&self, item: &SpellView) -> Option<bool> {
        let cast = item.cast?;
        if matches!(cast.method, CastMethod::Disguise | CastMethod::LibraryLife) {
            return None;
        }
        let counter = self.state.linked_choices.as_ref()?.library_entry_counter?;
        if counter.spell != item.id || counter.counter_faces & (1 << cast.face as u8) == 0 {
            return None;
        }
        if counter.ordinary_faces & (1 << cast.face as u8) != 0 {
            counter.selected
        } else {
            Some(true)
        }
    }

    pub(super) fn needs_library_entry_counter_choice(&self, item: &SpellView) -> bool {
        item.cast.is_some_and(|cast| {
            !matches!(cast.method, CastMethod::Disguise | CastMethod::LibraryLife)
                && self
                    .state
                    .linked_choices
                    .as_ref()
                    .and_then(|state| state.library_entry_counter)
                    .is_some_and(|counter| {
                        counter.spell == item.id
                            && counter.ordinary_faces & (1 << cast.face as u8) != 0
                            && counter.selected.is_none()
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
