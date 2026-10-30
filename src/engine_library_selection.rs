//! Private look-top selection; cards stay in place until the complete transaction commits.
use super::*;
use crate::cards::{
    LibraryLook, LibraryPickDestination, LibraryRemainder, LibrarySelectionCount,
    LibrarySelectionEffect, LibrarySelectionFilter, LibrarySelectionSpec,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibrarySelection {
    pub spec: LibrarySelectionSpec,
    pub owner: Player,
    pub may_transform: bool,
    pub entered: Option<ObjectRef>,
    pub transform_pending: bool,
    pub destination: LibraryPickDestination,
    /// The selected card is excluded from “the rest” even if its move is prevented.
    pub exclude_selected_from_remainder: bool,
    pub public: bool,
    pub entering: bool,
    pub remaining: Vec<ObjectRef>,
    pub eligible: Vec<ObjectRef>,
    pub selected: Vec<ObjectRef>,
    pub bottom: Vec<ObjectRef>,
    pub minimum: usize,
    pub maximum: usize,
    pub ordering: bool,
    pub random: bool,
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
    #[serde(skip_serializing_if = "is_false")]
    pub random: bool,
    pub reveal: bool,
    pub public: bool,
    pub destination: &'static str,
    pub remainder: &'static str,
}
fn is_false(value: &bool) -> bool {
    !value
}
#[derive(Clone, Debug, Serialize)]
pub struct EntryTransformChoiceView {
    pub object: ObjectView,
}
impl Game {
    fn library_selection_matches(&self, filter: LibrarySelectionFilter, id: CardId) -> bool {
        let c = self.state.objects[id as usize].card;
        match filter {
            LibrarySelectionFilter::HasX => c.x_symbols() > 0,
            LibrarySelectionFilter::Any => true,
            LibrarySelectionFilter::Kinds(mask) => c.kind_mask() & mask != 0,
            LibrarySelectionFilter::ExcludingKinds(mask) => c.kind_mask() & mask == 0,
            LibrarySelectionFilter::Subtype(subtype) => c.has_subtype(subtype),
            LibrarySelectionFilter::AnyOf(filters) => filters
                .iter()
                .any(|filter| self.library_selection_matches(*filter, id)),
            LibrarySelectionFilter::AllOf(filters) => filters
                .iter()
                .all(|filter| self.library_selection_matches(*filter, id)),
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
        self.start_library_selection_to(
            item,
            LibrarySelectionEffect {
                spec,
                look: LibraryLook::Fixed(spec.look),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            },
        )
    }
    pub(super) fn start_library_selection_to(
        &mut self,
        item: &SpellView,
        effect: LibrarySelectionEffect,
    ) -> bool {
        self.start_library_selection_completion(item, effect, false)
    }
    pub(super) fn start_library_selection_completion(
        &mut self,
        item: &SpellView,
        effect: LibrarySelectionEffect,
        may_transform: bool,
    ) -> bool {
        assert!(
            !may_transform
                || (effect.destination == LibraryPickDestination::Battlefield
                    && effect.spec.count == LibrarySelectionCount::Fixed(1))
        );
        let spec = effect.spec;
        let owner = item.controller;
        let limit = match effect.look {
            LibraryLook::Fixed(n) => usize::from(n),
            LibraryLook::Amount(value) => {
                usize::try_from(self.item_amount(value, item)).unwrap_or(usize::MAX)
            }
            LibraryLook::ChosenX => usize::try_from(item.x.unwrap_or(0)).unwrap_or(usize::MAX),
            LibraryLook::UntilKinds(kinds) => self.state.players[owner.index()]
                .library
                .iter()
                .rev()
                .filter(|&&id| self.state.objects[id as usize].is_card())
                .position(|&id| self.state.objects[id as usize].card.kind_mask() & kinds != 0)
                .map_or(usize::MAX, |at| at + 1),
        };
        let maximum_mana_value = effect.maximum_mana_value.map(|v| self.item_amount(v, item));
        let remaining: Vec<_> = self.state.players[owner.index()]
            .library
            .iter()
            .rev()
            .filter(|&&id| self.state.objects[id as usize].is_card())
            .take(limit)
            .map(|&id| self.object_ref(id))
            .collect();
        if remaining.is_empty() {
            return false;
        }
        let eligible: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|r| {
                self.library_selection_matches(spec.filter, r.card)
                    && maximum_mana_value.is_none_or(|max| {
                        u128::from(self.state.objects[r.card as usize].card.fixed_mana_value())
                            <= max
                    })
            })
            .collect();
        let destination = match effect.destination {
            LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(value) => {
                assert_eq!(spec.count, LibrarySelectionCount::Fixed(1));
                assert!(!spec.optional && eligible.len() <= 1);
                let maximum = self.item_amount(value, item);
                if eligible.first().is_some_and(|r| {
                    u128::from(self.state.objects[r.card as usize].card.fixed_mana_value())
                        <= maximum
                }) {
                    LibraryPickDestination::Battlefield
                } else {
                    LibraryPickDestination::Hand
                }
            }
            destination => destination,
        };
        let grave = || {
            self.state
                .objects
                .iter()
                .filter(|o| o.owner == owner && o.zone == Zone::Graveyard && o.is_card())
        };
        let requested = match spec.count {
            LibrarySelectionCount::Fixed(n) => usize::from(n),
            LibrarySelectionCount::RandomOne => 1,
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
            owner,
            may_transform,
            entered: None,
            transform_pending: false,
            destination,
            exclude_selected_from_remainder: matches!(
                effect.destination,
                LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(_)
            ) || matches!(effect.look, LibraryLook::UntilKinds(_)),
            public: effect.public,
            entering: false,
            remaining,
            eligible,
            selected: Vec::new(),
            bottom: Vec::new(),
            minimum,
            maximum,
            ordering: false,
            random: spec.count == LibrarySelectionCount::RandomOne,
        }));
        // Even a forced random remainder must expose the looked information
        // before it disappears into the library (there may be no eligible pick).
        if maximum == 0
            && !matches!(
                spec.remainder,
                LibraryRemainder::BottomRandom | LibraryRemainder::Shuffle
            )
        {
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
        if minimum == frame.remaining.len() && !frame.public {
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
        let count = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .selection
            .as_ref()
            .unwrap();
        let count = if count.random {
            count.eligible.len()
        } else {
            0
        };
        if count > 0 {
            let index = self.state.rng.below(count as u64) as usize;
            let card = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .eligible[index]
                .card;
            self.choose_library_selection(card);
        }
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
        self.forget_library_top(owner);
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
        if f.destination == LibraryPickDestination::LandsTappedRestHand {
            // The unchosen cards move first. During any ensuing entry choices,
            // only the selected, revealed cards remain in the public frame.
            self.commit_library_remainder(owner, (*f).clone());
            f.remaining.clear();
            f.bottom.clear();
            f.public = true;
            let moves: Vec<_> = f
                .selected
                .iter()
                .filter(|r| self.state.objects[r.card as usize].card.is_land())
                .map(|r| {
                    let mut movement = self.entry_move(r.card, Zone::Battlefield, true);
                    movement.controller = owner;
                    movement
                })
                .collect();
            if moves.is_empty() {
                self.library_selected_nonlands_to_hand(&f);
            } else {
                f.entering = true;
                self.state.resolving.as_mut().unwrap().selection = Some(f);
                self.stage_entries(moves, None);
            }
            return;
        }
        let ids: Vec<_> = f.selected.iter().map(|r| r.card).collect();
        let events = if f.destination == LibraryPickDestination::Hand {
            self.move_objects_raw(&ids, Zone::Hand)
        } else {
            Vec::new()
        };
        if f.destination == LibraryPickDestination::Hideaway {
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.remember_linked_exile(&ids);
            let source = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .item
                .source
                .expect("hideaway has source")
                .object;
            for event in &events {
                if event.moved && event.destination == Zone::Exile {
                    let data = self.state.linked_choices.get_or_insert_default();
                    let viewers = data
                        .hideaway_controllers
                        .iter()
                        .find(|(r, _)| *r == source)
                        .map_or(1 << owner.index(), |(_, mask)| *mask | (1 << owner.index()));
                    data.hidden.push((source, event.after, viewers));
                }
            }
            self.finish_zone_events(&events, &[]);
        }
        if f.spec.reveal {
            for e in &events {
                self.state.known_hand.push(e.after);
            }
        }
        self.finish_zone_events(&events, &[]);
        if f.destination == LibraryPickDestination::Top {
            let library = &mut self.state.players[owner.index()].library;
            library.retain(|id| !ids.contains(id));
            library.extend(ids.iter().rev());
        }
        if matches!(
            f.destination,
            LibraryPickDestination::Battlefield | LibraryPickDestination::BattlefieldTapped
        ) && !ids.is_empty()
        {
            f.entering = true;
            let tapped = f.destination == LibraryPickDestination::BattlefieldTapped;
            self.state.resolving.as_mut().unwrap().selection = Some(f);
            let moves = ids
                .into_iter()
                .map(|id| {
                    let mut movement = self.entry_move(id, Zone::Battlefield, tapped);
                    movement.controller = owner;
                    movement
                })
                .collect();
            self.stage_entries(moves, None);
            return;
        }
        self.commit_library_remainder(owner, *f);
    }
    fn library_selected_nonlands_to_hand(&mut self, frame: &LibrarySelection) {
        let ids: Vec<_> = frame
            .selected
            .iter()
            .filter(|r| {
                self.object_ref(r.card) == **r
                    && self.state.objects[r.card as usize].zone == Zone::Library
                    && !self.state.objects[r.card as usize].card.is_land()
            })
            .map(|r| r.card)
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Hand);
        if frame.spec.reveal {
            self.state.known_hand.extend(
                events
                    .iter()
                    .filter(|e| e.moved && e.destination == Zone::Hand)
                    .map(|e| e.after),
            );
        }
        self.finish_zone_events(&events, &[]);
    }
    fn commit_library_remainder(&mut self, owner: Player, mut f: LibrarySelection) {
        // Entry replacements may move another looked card. The remainder means
        // the original looked incarnations actually still in this library.
        let remains = |r: &ObjectRef| {
            self.object_ref(r.card) == *r
                && self.state.objects[r.card as usize].zone == Zone::Library
                && self.state.objects[r.card as usize].owner == owner
        };
        f.remaining.retain(remains);
        f.bottom.retain(remains);
        if !f.random
            && !f.exclude_selected_from_remainder
            && matches!(
                f.destination,
                LibraryPickDestination::Battlefield | LibraryPickDestination::BattlefieldTapped
            )
        {
            f.remaining.extend(f.selected.iter().copied().filter(|r| {
                self.object_ref(r.card) == *r
                    && self.state.objects[r.card as usize].zone == Zone::Library
            }));
        }
        match f.spec.remainder {
            LibraryRemainder::Shuffle => {
                if self.state.players[owner.index()].library.len() > 1 {
                    self.forget_library_top(owner);
                }
                self.shuffle_library_cards(owner);
            }
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
    pub(super) fn capture_selected_library_entry(&mut self, events: &[ZoneChange]) {
        let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .and_then(|r| r.selection.as_mut())
        else {
            return;
        };
        if frame.entering
            && frame.may_transform
            && let Some(event) = events.iter().find(|event| {
                event.moved
                    && event.destination == Zone::Battlefield
                    && frame.selected.contains(&event.before.reference)
            })
        {
            frame.entered = Some(event.after);
        }
    }
    pub(super) fn complete_library_entry_transform(&mut self, accept: bool) -> bool {
        if !self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.selection.as_ref())
            .is_some_and(|frame| frame.transform_pending)
        {
            return false;
        }
        let frame = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .take()
            .unwrap();
        if accept && let Some(recipient) = frame.entered {
            self.transform_permanent(recipient);
        }
        self.commit_library_remainder(frame.owner, *frame);
        true
    }
    pub(super) fn entry_transform_choice_view(&self) -> Option<EntryTransformChoiceView> {
        let frame = self.state.resolving.as_ref()?.selection.as_ref()?;
        if !frame.transform_pending {
            return None;
        }
        let object = frame.entered?;
        (self.on_battlefield(object.card) && self.object_ref(object.card) == object).then(|| {
            EntryTransformChoiceView {
                object: self.object_view(object.card),
            }
        })
    }
    pub(super) fn finish_entered_library_selection(&mut self) -> bool {
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .selection
            .as_ref()
            .is_some_and(|f| f.entering)
        {
            let recipient = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .entered;
            if recipient.is_some_and(|object| self.can_transform_permanent(object)) {
                let resolution = self.state.resolving.as_mut().unwrap();
                let frame = resolution.selection.as_mut().unwrap();
                if frame.may_transform {
                    frame.entering = false;
                    frame.transform_pending = true;
                    self.state.pending = Pending::EffectOptional {
                        player: frame.owner,
                        skip_to: resolution.pc,
                    };
                    return true;
                }
            }
            let resolution = self.state.resolving.as_mut().unwrap();
            let frame = resolution.selection.take().unwrap();
            let owner = frame.owner;
            if frame.destination == LibraryPickDestination::LandsTappedRestHand {
                self.library_selected_nonlands_to_hand(&frame);
            } else {
                self.commit_library_remainder(owner, *frame);
            }
        }
        false
    }
    pub(super) fn library_selection_view(&self, viewer: Player) -> Option<LibrarySelectionView> {
        let f = self.state.resolving.as_ref()?.selection.as_ref()?;
        if !self.shares_information(viewer, f.owner) && !f.public {
            return None;
        }
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
            random: f.random,
            reveal: f.spec.reveal,
            public: f.public,
            destination: match f.destination {
                LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(_) => {
                    unreachable!("normalized before frame creation")
                }
                LibraryPickDestination::LandsTappedRestHand => "lands_tapped_rest_hand",
                LibraryPickDestination::Hand => "hand",
                LibraryPickDestination::Battlefield => "battlefield",
                LibraryPickDestination::BattlefieldTapped => "battlefield_tapped",
                LibraryPickDestination::Top => "top",
                LibraryPickDestination::Hideaway => "hideaway",
            },
            remainder: match f.spec.remainder {
                LibraryRemainder::Graveyard => "graveyard",
                LibraryRemainder::Shuffle => "shuffle",
                LibraryRemainder::BottomAny => "bottom_any",
                LibraryRemainder::BottomRandom => "bottom_random",
            },
        })
    }
}
