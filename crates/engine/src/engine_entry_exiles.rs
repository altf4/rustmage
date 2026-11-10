//! Non-targeted owned entry selections; link only successful exile incarnations.
use super::*;
use crate::cards::SearchFilter;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Preparation {
    pub done: bool,
    pub selection_closed: bool,
    pub selected: Vec<ObjectRef>,
    pub exiled: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Preparation {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.selected.len() + self.exiled.len()) * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryExileChoiceView {
    pub entrant: ObjectRef,
    pub maximum: u32,
    pub selected: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn needs_exile(m: &entries::EntryMove) -> bool {
    m.destination == Zone::Battlefield
        && m.before.zone != Zone::Battlefield
        && !m.skipped
        && m.manifest_original.is_none()
        && m.intrinsic_entry_card()
            .is_some_and(|card| card.entry_exile_counters().is_some())
        && m.exile_preparation.as_ref().is_none_or(|p| !p.done)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn filter_matches<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    filter: SearchFilter,
    id: CardId,
) -> bool {
    match filter {
        SearchFilter::AnyOf(filters) => filters.iter().any(|&f| filter_matches(world, f, id)),
        SearchFilter::AllOf(filters) => filters.iter().all(|&f| filter_matches(world, f, id)),
        SearchFilter::NoAbilities if world.reference(id) == world.game().object_ref(id) => {
            !world.game().object_has_rules_abilities(id)
        }
        _ => filter.matches(world.definition(id)),
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn current_entry_exile_index(&self) -> Option<usize> {
        self.state
            .entry_batch
            .as_ref()?
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| needs_exile(m))
            .min_by_key(|(i, m)| (m.controller != self.state.active, *i))
            .map(|(i, _)| i)
    }
    pub(super) fn entry_exile_candidates(&self) -> Vec<ObjectRef> {
        let Some(index) = self.current_entry_exile_index() else {
            return Vec::new();
        };
        let batch = self.state.entry_batch.as_ref().unwrap();
        let m = &batch.moves[index];
        let selected = m
            .exile_preparation
            .as_ref()
            .map_or(&[][..], |p| p.selected.as_slice());
        if selected.len() >= m.x as usize {
            return Vec::new();
        }
        let spec = m.before.card.entry_exile_counters().unwrap();
        self.state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                let object = self.object_ref(id as CardId);
                (o.zone == Zone::Graveyard
                    && o.owner == m.controller
                    && o.is_card()
                    && filter_matches(self, spec.filter, id as CardId)
                    && !selected.contains(&object)
                    && !batch.moves.iter().any(|other| {
                        other
                            .exile_preparation
                            .as_ref()
                            .is_some_and(|p| p.selected.contains(&object))
                    })
                    && !batch.moves.iter().any(|other| {
                        !other.skipped
                            && other.destination == Zone::Battlefield
                            && other.before.reference == object
                    }))
                .then_some(object)
            })
            .collect()
    }
    pub(super) fn advance_entry_exiles(&mut self) -> bool {
        loop {
            let Some(index) = self.current_entry_exile_index() else {
                return false;
            };
            if !self.state.entry_batch.as_ref().unwrap().moves[index]
                .exile_preparation
                .as_ref()
                .is_some_and(|p| p.selection_closed)
                && !self.entry_exile_candidates().is_empty()
            {
                self.state.pending = Pending::EntryExile {
                    player: self.state.entry_batch.as_ref().unwrap().moves[index].controller,
                };
                return true;
            }
            self.finish_entry_exile();
        }
    }
    pub(super) fn choose_entry_exile(&mut self, object: ObjectRef) {
        let index = self.current_entry_exile_index().unwrap();
        self.state.entry_batch.as_mut().unwrap().moves[index]
            .exile_preparation
            .get_or_insert_default()
            .selected
            .push(object);
    }
    pub(super) fn finish_entry_exile(&mut self) {
        let index = self.current_entry_exile_index().unwrap();
        let selected = self.state.entry_batch.as_ref().unwrap().moves[index]
            .exile_preparation
            .as_ref()
            .map_or_else(Vec::new, |p| p.selected.clone());
        let ids = selected
            .iter()
            .filter(|r| {
                self.object_ref(r.card) == **r
                    && self.state.objects[r.card as usize].zone == Zone::Graveyard
            })
            .map(|r| r.card)
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let exiled = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile)
            .map(|e| e.after)
            .collect();
        let prep = self.state.entry_batch.as_mut().unwrap().moves[index]
            .exile_preparation
            .get_or_insert_default();
        prep.done = true;
        prep.exiled = exiled;
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn entry_exile_view(&self, viewer: Player) -> Option<EntryExileChoiceView> {
        if !matches!(self.state.pending, Pending::EntryExile { .. }) {
            return None;
        }
        let m = &self.state.entry_batch.as_ref()?.moves[self.current_entry_exile_index()?];
        if !entries::visible_mask(m, self.information_mask(viewer), &self.state.known_hand) {
            return None;
        }
        Some(EntryExileChoiceView {
            entrant: m.before.reference,
            maximum: m.x,
            selected: m
                .exile_preparation
                .as_ref()
                .map_or_else(Vec::new, |p| p.selected.clone()),
            candidates: self.entry_exile_candidates(),
        })
    }
}
