//! Qualified hidden-zone search; the chosen library is shuffled even on failure to find.
use super::*;
use crate::SearchFilter;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandLibrarySearch {
    player: Player,
    filter: SearchFilter,
    zones: Option<(bool, bool)>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<ObjectView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn start_hand_library_search(&mut self, player: Player, filter: SearchFilter) {
        self.state
            .linked_choices
            .get_or_insert_default()
            .hand_library_search = Some(Box::new(HandLibrarySearch {
            player,
            filter,
            zones: None,
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
        let Some((hand, library)) = s.zones else {
            return false;
        };
        o.is_card()
            && o.owner == s.player
            && ((hand && o.zone == Zone::Hand) || (library && o.zone == Zone::Library))
    }
    fn hand_library_selected(&self, s: &HandLibrarySearch) -> Option<ObjectRef> {
        s.selected.filter(|r| {
            self.hand_library_zone(s, r.card)
                && self.object_ref(r.card) == *r
                && s.filter.matches(self.state.objects[r.card as usize].card)
        })
    }
    pub(super) fn visit_hand_library_search(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let s = self.hand_library_search().unwrap();
        if s.zones.is_none() {
            for (hand, library) in [(true, false), (false, true), (true, true)] {
                if emit(Action::ChooseHandLibrarySearchZones { hand, library }) {
                    return true;
                }
            }
            return false;
        }
        // CR701.23b: both zones are hidden, so finding zero is always legal.
        if emit(Action::FinishSearch) {
            return true;
        }
        for (id, o) in self.state.objects.iter().enumerate() {
            let id = id as CardId;
            if self.hand_library_zone(s, id)
                && s.filter.matches(o.card)
                && s.selected != Some(self.object_ref(id))
                && emit(Action::ChooseSearch { card: id })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn choose_hand_library_zones(&mut self, hand: bool, library: bool) {
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .hand_library_search
            .as_mut()
            .unwrap()
            .zones = Some((hand, library));
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
        if s.zones.unwrap().1 {
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
        self.stage_entries(moves, s.zones.unwrap().1.then_some(s.player));
        self.run_program();
    }
    pub(super) fn hand_library_search_view(&self, viewer: Player) -> Option<HandLibrarySearchView> {
        let s = self.hand_library_search()?;
        let (hand, library) = s.zones.unwrap_or((false, false));
        let entitled = self.shares_information(viewer, s.player);
        Some(HandLibrarySearchView {
            player: s.player,
            filter: s.filter,
            choosing_zones: s.zones.is_none(),
            hand,
            library,
            cards: (entitled && s.zones.is_some()).then(|| {
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
