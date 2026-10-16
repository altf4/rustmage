//! Private look-top selection; cards stay in place until the complete transaction commits.
use super::*;
use crate::cards::{
    LibraryRemainder, LibrarySelectionCount, LibrarySelectionFilter, LibrarySelectionSpec,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibrarySelection {
    pub spec: LibrarySelectionSpec,
    pub remaining: Vec<ObjectRef>,
    pub eligible: Vec<ObjectRef>,
    pub selected: Vec<ObjectRef>,
    pub bottom: Vec<ObjectRef>,
    pub minimum: usize,
    pub maximum: usize,
    pub ordering: bool,
}
impl LibrarySelection {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.remaining.len() + self.eligible.len() + self.selected.len() + self.bottom.len())
                * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct LibrarySelectionView {
    pub remaining: Vec<ObjectView>,
    pub candidates: Vec<ObjectRef>,
    pub selected: Vec<ObjectView>,
    pub bottom: Vec<ObjectView>,
    pub minimum: usize,
    pub maximum: usize,
    pub ordering: bool,
    pub reveal: bool,
    pub remainder: &'static str,
}
impl Game {
    fn library_selection_matches(&self, filter: LibrarySelectionFilter, id: CardId) -> bool {
        let c = self.state.objects[id as usize].card;
        match filter {
            LibrarySelectionFilter::Any => true,
            LibrarySelectionFilter::Kinds(mask) => c.kind_mask() & mask != 0,
            LibrarySelectionFilter::CreaturePowerAtMost(power) => {
                c.is_creature() && self.stats(id).power <= i128::from(power)
            }
        }
    }
    pub(super) fn start_library_selection(
        &mut self,
        item: &SpellView,
        spec: LibrarySelectionSpec,
    ) -> bool {
        let owner = item.controller;
        let remaining: Vec<_> = self.state.players[owner.index()]
            .library
            .iter()
            .rev()
            .take(usize::from(spec.look))
            .map(|&id| self.object_ref(id))
            .collect();
        if remaining.is_empty() {
            return false;
        }
        let eligible: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|r| self.library_selection_matches(spec.filter, r.card))
            .collect();
        let grave = || {
            self.state
                .objects
                .iter()
                .filter(|o| o.owner == owner && o.zone == Zone::Graveyard && !o.card.is_token())
        };
        let requested = match spec.count {
            LibrarySelectionCount::Fixed(n) => usize::from(n),
            LibrarySelectionCount::Lessons => {
                if grave().filter(|o| o.card.has_subtype("Lesson")).count() >= 3 {
                    remaining.len()
                } else {
                    1
                }
            }
            LibrarySelectionCount::InstantAndSorcery => {
                if grave().any(|o| o.card.is_instant())
                    && grave().any(|o| o.card.kind_mask() & 4 != 0)
                {
                    2
                } else {
                    1
                }
            }
            LibrarySelectionCount::GainedLife => {
                if self.state.players[owner.index()].life_gained_this_turn > 0 {
                    2
                } else {
                    1
                }
            }
        };
        let maximum = requested.min(eligible.len());
        let minimum = if spec.optional { 0 } else { maximum };
        self.state.resolving.as_mut().unwrap().selection = Some(Box::new(LibrarySelection {
            spec,
            remaining,
            eligible,
            selected: Vec::new(),
            bottom: Vec::new(),
            minimum,
            maximum,
            ordering: false,
        }));
        // Even a forced random remainder must expose the looked information
        // before it disappears into the library (there may be no eligible pick).
        if maximum == 0 && spec.remainder != LibraryRemainder::BottomRandom {
            return self.finish_library_selection_internal(owner);
        }
        let frame = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap();
        if minimum == frame.remaining.len() {
            frame.selected = std::mem::take(&mut frame.remaining);
            return self.finish_library_selection_internal(owner);
        }
        self.state.pending = Pending::EffectLibrarySelection { player: owner };
        true
    }
    pub(super) fn choose_library_selection(&mut self, card: CardId) {
        let f = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap();
        let index = f.remaining.iter().position(|r| r.card == card).unwrap();
        f.selected.push(f.remaining.remove(index));
    }
    pub(super) fn order_library_bottom(&mut self, owner: Player, card: CardId) {
        let f = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap();
        let index = f.remaining.iter().position(|r| r.card == card).unwrap();
        f.bottom.push(f.remaining.remove(index));
        if f.remaining.is_empty() {
            self.commit_library_selection(owner);
            self.run_program();
        }
    }
    pub(super) fn finish_library_selection(&mut self, owner: Player) {
        if !self.finish_library_selection_internal(owner) {
            self.run_program();
        }
    }
    fn finish_library_selection_internal(&mut self, owner: Player) -> bool {
        let f = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap();
        if f.spec.remainder == LibraryRemainder::BottomAny && !f.remaining.is_empty() {
            f.ordering = true;
            self.state.pending = Pending::EffectLibrarySelection { player: owner };
            true
        } else {
            self.commit_library_selection(owner);
            false
        }
    }
    fn commit_library_selection(&mut self, owner: Player) {
        let mut f = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .take()
            .unwrap();
        for reference in f.selected.iter().chain(&f.remaining).chain(&f.bottom) {
            assert_eq!(self.object_ref(reference.card), *reference);
            assert_eq!(
                self.state.objects[reference.card as usize].zone,
                Zone::Library
            );
        }
        let ids: Vec<_> = f.selected.iter().map(|r| r.card).collect();
        let events = self.move_objects_raw(&ids, Zone::Hand);
        if f.spec.reveal {
            for e in &events {
                self.state.known_hand.push(e.after);
            }
        }
        self.finish_zone_events(&events, &[]);
        match f.spec.remainder {
            LibraryRemainder::Graveyard => {
                let ids: Vec<_> = f.remaining.iter().map(|r| r.card).collect();
                let events = self.move_objects_raw(&ids, Zone::Graveyard);
                self.finish_zone_events(&events, &[]);
            }
            LibraryRemainder::BottomAny | LibraryRemainder::BottomRandom => {
                if f.spec.remainder == LibraryRemainder::BottomRandom {
                    f.bottom = std::mem::take(&mut f.remaining);
                    self.state.rng.shuffle(&mut f.bottom);
                }
                let library = &mut self.state.players[owner.index()].library;
                library.retain(|id| !f.bottom.iter().any(|r| r.card == *id));
                library.splice(0..0, f.bottom.iter().rev().map(|r| r.card));
            }
        }
    }
    pub(super) fn library_selection_view(&self, viewer: Player) -> Option<LibrarySelectionView> {
        if self.actor() != Some(viewer) {
            return None;
        }
        let f = self.state.resolving.as_ref()?.selection.as_ref()?;
        Some(LibrarySelectionView {
            remaining: f
                .remaining
                .iter()
                .map(|r| self.object_view(r.card))
                .collect(),
            candidates: f
                .remaining
                .iter()
                .copied()
                .filter(|r| f.eligible.contains(r))
                .collect(),
            selected: f
                .selected
                .iter()
                .map(|r| self.object_view(r.card))
                .collect(),
            bottom: f.bottom.iter().map(|r| self.object_view(r.card)).collect(),
            minimum: f.minimum,
            maximum: f.maximum,
            ordering: f.ordering,
            reveal: f.spec.reveal,
            remainder: match f.spec.remainder {
                LibraryRemainder::Graveyard => "graveyard",
                LibraryRemainder::BottomAny => "bottom_any",
                LibraryRemainder::BottomRandom => "bottom_random",
            },
        })
    }
}
