//! Owned library/graveyard search with one captured X and exact entry completion.
use super::*;
use crate::SearchFilter;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ZoneSearch {
    player: Player,
    source: ObjectRef,
    filter: SearchFilter,
    x: u32,
    counter: CounterKind,
    haste: bool,
    zones: Option<(bool, bool)>,
    selected: Option<ObjectRef>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ZoneSearchView {
    pub player: Player,
    pub filter: SearchFilter,
    pub choosing_zones: bool,
    pub library: bool,
    pub graveyard: bool,
    pub maximum_mana_value: u64,
    pub counter: CounterKind,
    pub counter_amount: u64,
    pub haste: bool,
    pub minimum: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<ObjectView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<ObjectRef>,
}
impl Game {
    pub(super) fn start_zone_search(
        &mut self,
        item: &SpellView,
        filter: SearchFilter,
        counter: CounterKind,
        haste_at: Option<u32>,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let x = item.x.unwrap_or(0);
        self.state
            .linked_choices
            .get_or_insert_default()
            .zone_search = Some(Box::new(ZoneSearch {
            player: item.controller,
            source: source.object,
            filter,
            x,
            counter,
            haste: haste_at.is_some_and(|n| x >= n),
            zones: None,
            selected: None,
        }));
        self.state.pending = Pending::LibraryGraveyardSearch {
            player: item.controller,
        };
    }
    fn zone_search(&self) -> Option<&ZoneSearch> {
        self.state.linked_choices.as_ref()?.zone_search.as_deref()
    }
    fn zone_search_candidate(&self, s: &ZoneSearch, id: CardId) -> bool {
        let Some(o) = self.state.objects.get(id as usize) else {
            return false;
        };
        let Some((library, graveyard)) = s.zones else {
            return false;
        };
        !o.is_token()
            && o.owner == s.player
            && ((library && o.zone == Zone::Library) || (graveyard && o.zone == Zone::Graveyard))
            && s.filter.matches(o.card)
            && o.card.fixed_mana_value() <= u64::from(s.x)
    }
    fn zone_search_selected(&self, s: &ZoneSearch) -> Option<ObjectRef> {
        s.selected
            .filter(|r| self.object_ref(r.card) == *r && self.zone_search_candidate(s, r.card))
    }
    fn zone_search_minimum(&self, s: &ZoneSearch) -> u16 {
        // CR701.23b permits failing a hidden qualified search, not a public one.
        u16::from(
            s.zones.is_some_and(|(_, gy)| gy)
                && self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.zone == Zone::Graveyard && self.zone_search_candidate(s, id as CardId)
                }),
        )
    }
    pub(super) fn visit_zone_search(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let s = self.zone_search().unwrap();
        if s.zones.is_none() {
            for (library, graveyard) in [(true, false), (false, true), (true, true)] {
                if emit(Action::ChooseSearchZones { library, graveyard }) {
                    return true;
                }
            }
            return false;
        }
        if (self.zone_search_selected(s).is_some() || self.zone_search_minimum(s) == 0)
            && emit(Action::FinishSearch)
        {
            return true;
        }
        for (id, _) in self.state.objects.iter().enumerate() {
            if self.zone_search_candidate(s, id as CardId)
                && s.selected != Some(self.object_ref(id as CardId))
                && emit(Action::ChooseSearch { card: id as CardId })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn choose_search_zones(&mut self, library: bool, graveyard: bool) {
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .zone_search
            .as_mut()
            .unwrap()
            .zones = Some((library, graveyard));
    }
    pub(super) fn has_zone_search(&self) -> bool {
        self.zone_search().is_some()
    }
    pub(super) fn choose_zone_search_card(&mut self, id: CardId) {
        let reference = self.object_ref(id);
        self.state
            .linked_choices
            .as_mut()
            .unwrap()
            .zone_search
            .as_mut()
            .unwrap()
            .selected = Some(reference);
    }
    pub(super) fn finish_zone_search(&mut self) {
        let selected = self.zone_search_selected(self.zone_search().unwrap());
        let s = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .zone_search
            .take()
            .unwrap();
        let moves = selected
            .into_iter()
            .map(|r| {
                let mut m = self.entry_move(r.card, Zone::Battlefield, false);
                m.effect_counter = Some((s.counter, u64::from(s.x)));
                if s.haste {
                    m.keywords_after = Some((s.source, Keywords::HASTE));
                }
                m
            })
            .collect();
        self.stage_entries(moves, s.zones.unwrap().0.then_some(s.player));
        self.run_program();
    }
    pub(super) fn zone_search_view(&self, viewer: Player) -> Option<ZoneSearchView> {
        let s = self.zone_search()?;
        let (library, graveyard) = s.zones.unwrap_or((false, false));
        let entitled = self.shares_information(viewer, s.player);
        Some(ZoneSearchView {
            player: s.player,
            filter: s.filter,
            choosing_zones: s.zones.is_none(),
            library,
            graveyard,
            maximum_mana_value: u64::from(s.x),
            counter: s.counter,
            counter_amount: u64::from(s.x),
            haste: s.haste,
            minimum: self.zone_search_minimum(s),
            cards: (entitled && s.zones.is_some()).then(|| {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        !o.is_token()
                            && o.owner == s.player
                            && ((library && o.zone == Zone::Library)
                                || (graveyard && o.zone == Zone::Graveyard))
                    })
                    .map(|(id, _)| self.object_view(id as CardId))
                    .collect()
            }),
            selected: entitled.then(|| self.zone_search_selected(s)).flatten(),
        })
    }
}
