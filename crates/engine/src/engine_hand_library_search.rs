//! Qualified search of selected zones, with one exact battlefield result.
use super::*;
use crate::SearchFilter;

const HAND: u8 = 1;
const LIBRARY: u8 = 2;
const GRAVEYARD: u8 = 4;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandLibrarySearch {
    player: Player,
    filter: SearchFilter,
    /// The zones actually searched. The three-zone route can add to this set.
    zones: u8,
    include_graveyard: bool,
    selected: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct HandLibrarySearchView {
    pub player: Player,
    pub filter: SearchFilter,
    pub choosing_zones: bool,
    pub hand: bool,
    pub library: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub graveyard: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<ObjectView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn start_hand_library_search(
        &mut self,
        player: Player,
        filter: SearchFilter,
        include_graveyard: bool,
    ) {
        self.state
            .linked_choices
            .get_or_insert_default()
            .hand_library_search = Some(Box::new(HandLibrarySearch {
            player,
            filter,
            zones: 0,
            include_graveyard,
            selected: None,
        }));
        self.state.pending = Pending::HandLibrarySearch { player };
    }
    fn hand_library_search(&self) -> Option<&HandLibrarySearch> {
        self.state
            .linked_choices
            .as_ref()?
            .hand_library_search
            .as_deref()
    }
    pub(super) fn has_hand_library_search(&self) -> bool {
        self.hand_library_search().is_some()
    }
    fn hand_library_zone(&self, s: &HandLibrarySearch, id: CardId) -> bool {
        let Some(o) = self.state.objects.get(id as usize) else {
            return false;
        };
        o.is_card()
            && o.owner == s.player
            && match o.zone {
                Zone::Hand => s.zones & HAND != 0,
                Zone::Library => s.zones & LIBRARY != 0,
                Zone::Graveyard => s.zones & GRAVEYARD != 0,
                _ => false,
            }
    }
    fn hand_library_candidate(&self, s: &HandLibrarySearch, id: CardId) -> bool {
        self.hand_library_zone(s, id) && self.card_search_filter_matches(s.filter, s.player, id)
    }
    fn hand_library_selected(&self, s: &HandLibrarySearch) -> Option<ObjectRef> {
        s.selected
            .filter(|r| self.hand_library_candidate(s, r.card) && self.object_ref(r.card) == *r)
    }
    fn hand_library_minimum(&self, s: &HandLibrarySearch) -> u16 {
        // CR701.23b allows failure to find in qualified hidden-zone searches.
        u16::from(
            s.zones & GRAVEYARD != 0
                && self.state.objects.iter().enumerate().any(|(id, object)| {
                    object.zone == Zone::Graveyard && self.hand_library_candidate(s, id as CardId)
                }),
        )
    }
    pub(super) fn visit_hand_library_search(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let s = self.hand_library_search().unwrap();
        if s.include_graveyard {
            for zones in 1..=(HAND | LIBRARY | GRAVEYARD) {
                if zones != s.zones
                    && zones & s.zones == s.zones
                    && emit(Action::ChooseGraveyardHandLibrarySearchZones {
                        graveyard: zones & GRAVEYARD != 0,
                        hand: zones & HAND != 0,
                        library: zones & LIBRARY != 0,
                    })
                {
                    return true;
                }
            }
        } else if s.zones == 0 {
            for (hand, library) in [(true, false), (false, true), (true, true)] {
                if emit(Action::ChooseHandLibrarySearchZones { hand, library }) {
                    return true;
                }
            }
        }
        if s.zones == 0 {
            return false;
        }
        if (self.hand_library_selected(s).is_some() || self.hand_library_minimum(s) == 0)
            && emit(Action::FinishSearch)
        {
            return true;
        }
        for (id, _) in self.state.objects.iter().enumerate() {
            let id = id as CardId;
            if self.hand_library_candidate(s, id)
                && s.selected != Some(self.object_ref(id))
                && emit(Action::ChooseSearch { card: id })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn choose_hand_library_zones(&mut self, hand: bool, library: bool, graveyard: bool) {
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .hand_library_search
            .as_mut()
            .unwrap()
            .zones = (u8::from(hand) * HAND)
            | (u8::from(library) * LIBRARY)
            | (u8::from(graveyard) * GRAVEYARD);
    }
    pub(super) fn choose_hand_library_card(&mut self, id: CardId) {
        let object = self.object_ref(id);
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .hand_library_search
            .as_mut()
            .unwrap()
            .selected = Some(object);
    }
    pub(super) fn finish_hand_library_search(&mut self) {
        let selected = self.hand_library_selected(self.hand_library_search().unwrap());
        let s = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .hand_library_search
            .take()
            .unwrap();
        let searched_library = s.zones & LIBRARY != 0;
        if searched_library {
            self.library_searched(s.player);
        }
        let moves = selected
            .into_iter()
            // CR304.4/307.4: a found instant/sorcery cannot enter; it stays hidden.
            .filter(|r| {
                self.state.objects[r.card as usize].card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256)
                    != 0
            })
            .map(|r| self.entry_move(r.card, Zone::Battlefield, false))
            .collect();
        self.stage_entries(moves, searched_library.then_some(s.player));
        self.run_program();
    }
    pub(super) fn hand_library_search_view(&self, viewer: Player) -> Option<HandLibrarySearchView> {
        let s = self.hand_library_search()?;
        let entitled = self.shares_information(viewer, s.player);
        Some(HandLibrarySearchView {
            player: s.player,
            filter: s.filter,
            choosing_zones: s.zones == 0,
            hand: s.zones & HAND != 0,
            library: s.zones & LIBRARY != 0,
            graveyard: s.zones & GRAVEYARD != 0,
            minimum: s.include_graveyard.then(|| self.hand_library_minimum(s)),
            cards: (entitled && s.zones != 0).then(|| {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| self.hand_library_zone(s, *id as CardId))
                    .map(|(id, _)| self.object_view(id as CardId))
                    .collect()
            }),
            selected: entitled.then(|| self.hand_library_selected(s)).flatten(),
        })
    }
}
