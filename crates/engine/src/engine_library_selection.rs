//! Owned library selections, including public sequential picks.
use super::*;
use crate::cards::{
    LibraryLook, LibraryPickDestination, LibraryRemainder, LibrarySelectionCount,
    LibrarySelectionEffect, LibrarySelectionFilter, LibrarySelectionSpec,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TopLandStage {
    Hand,
    Graveyard,
    HandThenLands,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlternatingLibraryPick {
    Hand,
    Bottom,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AlternatingLibraryPicks {
    next: AlternatingLibraryPick,
    steps_left: u16,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LibraryEntryCompletion {
    None,
    MayTransform,
    ReflexiveEquipment(u16),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibrarySelection {
    alternating: Option<AlternatingLibraryPicks>,
    pub top_land: Option<TopLandStage>,
    pub spec: LibrarySelectionSpec,
    pub owner: Player,
    completion: LibraryEntryCompletion,
    entered: Vec<ObjectRef>,
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
    pub available_colors: Option<u8>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl LibrarySelection {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.remaining.len()
                + self.eligible.len()
                + self.selected.len()
                + self.bottom.len()
                + self.entered.len())
                * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    pub available_colors: Option<u8>,
    pub reveal: bool,
    pub public: bool,
    pub destination: &'static str,
    pub remainder: &'static str,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !value
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct EntryTransformChoiceView {
    pub object: ObjectView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn library_selection_matches(
        &self,
        filter: LibrarySelectionFilter,
        id: CardId,
        chosen_type: Option<crate::cards::CreatureType>,
    ) -> bool {
        let c = self.state.objects[id as usize].card;
        match filter {
            LibrarySelectionFilter::SourceChosenCreatureType => {
                chosen_type.is_some_and(|kind| c.has_subtype(kind.name()))
            }
            LibrarySelectionFilter::HasX => c.x_symbols() > 0,
            LibrarySelectionFilter::Any => true,
            LibrarySelectionFilter::Kinds(mask) => c.kind_mask() & mask != 0,
            LibrarySelectionFilter::ExcludingKinds(mask) => c.kind_mask() & mask == 0,
            LibrarySelectionFilter::Subtype(subtype) => c.has_subtype(subtype),
            LibrarySelectionFilter::AnyOf(filters) => filters
                .iter()
                .any(|filter| self.library_selection_matches(*filter, id, chosen_type)),
            LibrarySelectionFilter::AllOf(filters) => filters
                .iter()
                .all(|filter| self.library_selection_matches(*filter, id, chosen_type)),
            LibrarySelectionFilter::CreaturePowerAtMost(power) => {
                c.is_creature() && self.stats(id).power <= i128::from(power)
            }
        }
    }
    pub(super) fn start_top_land_then_graveyard(&mut self, item: &SpellView) -> bool {
        let owner = item.controller;
        let Some(id) = self.top_library_card(owner) else {
            return false;
        };
        let object = self.object_ref(id);
        let land = self.current_kinds(id) & 2 != 0;
        self.remember_private_library_top(owner, owner, object);
        self.state.resolving.as_mut().unwrap().selection = Some(Box::new(LibrarySelection {
            // Ineligible hidden cards still take the same first decision. The
            // opponent must not infer landness from public resolution pacing.
            alternating: None,
            top_land: Some(TopLandStage::Hand),
            spec: LibrarySelectionSpec {
                look: 1,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: true,
                reveal: land,
                remainder: LibraryRemainder::Keep,
            },
            owner,
            completion: LibraryEntryCompletion::None,
            entered: Vec::new(),
            transform_pending: false,
            destination: LibraryPickDestination::Hand,
            exclude_selected_from_remainder: true,
            public: false,
            entering: false,
            remaining: vec![object],
            eligible: if land { vec![object] } else { Vec::new() },
            selected: Vec::new(),
            bottom: Vec::new(),
            minimum: 0,
            maximum: 1,
            ordering: false,
            random: false,
            available_colors: None,
        }));
        self.state.pending = Pending::EffectLibrarySelection { player: owner };
        true
    }
    pub(super) fn start_top_hand_then_lands(
        &mut self,
        item: &SpellView,
        amount: AmountValue,
    ) -> bool {
        let pending = self.start_library_selection_to(
            item,
            LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Any,
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::Keep,
                },
                look: LibraryLook::Amount(amount),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            },
        );
        if pending {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .selection
                .as_mut()
                .unwrap()
                .top_land = Some(TopLandStage::HandThenLands);
        }
        pending
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
        self.start_library_selection_completion(item, effect, LibraryEntryCompletion::None)
    }
    pub(super) fn start_library_selection_completion(
        &mut self,
        item: &SpellView,
        effect: LibrarySelectionEffect,
        completion: LibraryEntryCompletion,
    ) -> bool {
        assert!(
            completion == LibraryEntryCompletion::None
                || effect.destination == LibraryPickDestination::Battlefield
        );
        assert!(
            completion != LibraryEntryCompletion::MayTransform
                || effect.spec.count == LibrarySelectionCount::Fixed(1)
        );
        let spec = effect.spec;
        let owner = item.controller;
        let limit = match effect.look {
            LibraryLook::Fixed(n) => usize::from(n),
            LibraryLook::Amount(value) => {
                usize::try_from(self.item_amount(value, item)).unwrap_or(usize::MAX)
            }
            LibraryLook::ChosenX => usize::try_from(item.x.unwrap_or(0)).unwrap_or(usize::MAX),
            LibraryLook::UntilCount { amount, .. } | LibraryLook::UntilNonlandCount(amount) => {
                let requested = self.item_amount(amount, item);
                if requested == 0 {
                    0
                } else {
                    let mut count = 0u128;
                    self.state.players[owner.index()]
                        .library
                        .iter()
                        .rev()
                        .filter(|&&id| self.state.objects[id as usize].is_card())
                        .position(|&id| {
                            let card = self.state.objects[id as usize].card;
                            let matches = match effect.look {
                                LibraryLook::UntilCount { kinds, .. } => {
                                    card.kind_mask() & kinds != 0
                                }
                                _ => !card.is_land(),
                            };
                            if matches {
                                count += 1;
                            }
                            count >= requested
                        })
                        .map_or(usize::MAX, |at| at + 1)
                }
            }
            LibraryLook::UntilKindsCount { kinds, count } => {
                if count == 0 {
                    0
                } else {
                    self.state.players[owner.index()]
                        .library
                        .iter()
                        .rev()
                        .filter(|&&id| self.state.objects[id as usize].is_card())
                        .enumerate()
                        .filter(|(_, id)| {
                            self.state.objects[**id as usize].card.kind_mask() & kinds != 0
                        })
                        .nth(usize::from(count - 1))
                        .map_or(usize::MAX, |(at, _)| at + 1)
                }
            }
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
            if spec.remainder == LibraryRemainder::Shuffle {
                if self.state.players[owner.index()].library.len() > 1 {
                    self.forget_library_top(owner);
                }
                self.shuffle_library_cards(owner);
            }
            return false;
        }
        let eligible: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|r| {
                self.library_selection_matches(
                    spec.filter,
                    r.card,
                    self.source_chosen_creature_type(item),
                ) && maximum_mana_value.is_none_or(|max| {
                    u128::from(self.state.objects[r.card as usize].card.fixed_mana_value()) <= max
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
        let available_colors = (spec.count == LibrarySelectionCount::ControlledColors).then(|| {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, object)| {
                    object.zone == Zone::Battlefield && object.controller == owner
                })
                .fold(0u8, |colors, (id, _)| {
                    colors | self.object_view(id as CardId).colors
                })
                & 31
        });
        let requested = match spec.count {
            LibrarySelectionCount::ControlledColors => {
                available_colors.unwrap().count_ones() as usize
            }
            LibrarySelectionCount::Fixed(n) => usize::from(n),
            LibrarySelectionCount::ManaValueBudget { maximum_cards, .. } => {
                assert!(spec.optional, "budgeted selection requires an up-to count");
                usize::from(maximum_cards)
            }
            LibrarySelectionCount::All => eligible.len(),
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
            alternating: None,
            top_land: None,
            spec,
            owner,
            completion,
            entered: Vec::new(),
            transform_pending: false,
            destination,
            exclude_selected_from_remainder: matches!(
                effect.destination,
                LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(_)
            ) || matches!(
                effect.look,
                LibraryLook::UntilKinds(_) | LibraryLook::UntilKindsCount { .. }
            ),
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
            available_colors,
        }));
        // Even a forced random remainder must expose the looked information
        // before it disappears into the library (there may be no eligible pick).
        if maximum == 0
            && !matches!(
                spec.remainder,
                LibraryRemainder::BottomRandom
                    | LibraryRemainder::Shuffle
                    | LibraryRemainder::OptionalGraveyard
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
    pub(super) fn start_alternating_library_picks(&mut self, item: &SpellView, look: u16) -> bool {
        if !self.start_library_selection_to(
            item,
            LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Any,
                    optional: false,
                    reveal: true,
                    remainder: LibraryRemainder::Keep,
                },
                look: LibraryLook::Fixed(look),
                destination: LibraryPickDestination::Hand,
                public: true,
                maximum_mana_value: None,
            },
        ) {
            return false;
        }
        let resolution = self.state.resolving.as_mut().unwrap();
        resolution.selection.as_mut().unwrap().alternating = Some(AlternatingLibraryPicks {
            next: AlternatingLibraryPick::Hand,
            steps_left: look,
        });
        self.advance_alternating_library_picks()
    }

    fn advance_alternating_library_picks(&mut self) -> bool {
        loop {
            let (owner, sequence, remaining, known_top) = {
                let objects = &self.state.objects;
                let resolution = self.state.resolving.as_mut().unwrap();
                let frame = resolution.selection.as_mut().unwrap();
                let owner = frame.owner;
                frame
                    .remaining
                    .retain(|&object| exact_library_card(objects, owner, object));
                frame
                    .bottom
                    .retain(|&object| exact_library_card(objects, owner, object));
                (
                    owner,
                    frame.alternating.unwrap(),
                    frame.remaining.len(),
                    frame
                        .remaining
                        .first()
                        .or_else(|| frame.bottom.first())
                        .copied(),
                )
            };
            if let Some(top) = known_top
                && self.top_library_card(owner) == Some(top.card)
            {
                self.remember_library_top(owner, top);
            }
            match (sequence.steps_left, remaining) {
                (0, _) | (_, 0) => {
                    self.state.resolving.as_mut().unwrap().selection = None;
                    return false;
                }
                (_, 1) => self.commit_alternating_library_pick(known_top.unwrap().card),
                _ => {
                    self.state.pending = Pending::EffectLibrarySelection {
                        player: match sequence.next {
                            AlternatingLibraryPick::Hand => owner,
                            AlternatingLibraryPick::Bottom => owner.other(),
                        },
                    };
                    return true;
                }
            }
        }
    }

    fn commit_alternating_library_pick(&mut self, card: CardId) {
        let (owner, object, pick) = {
            let resolution = self.state.resolving.as_mut().unwrap();
            let frame = resolution.selection.as_mut().unwrap();
            let index = frame.remaining.iter().position(|r| r.card == card).unwrap();
            let object = frame.remaining[index];
            let sequence = frame.alternating.as_mut().unwrap();
            let pick = sequence.next;
            sequence.steps_left = if frame.remaining.len() == 1 {
                0
            } else {
                sequence.steps_left - 1
            };
            sequence.next = match pick {
                // The next step prunes actual departures. A prevented hand
                // move leaves the same revealed card eligible for another pick.
                AlternatingLibraryPick::Hand => AlternatingLibraryPick::Bottom,
                AlternatingLibraryPick::Bottom => {
                    frame.remaining.remove(index);
                    frame.bottom.push(object);
                    AlternatingLibraryPick::Hand
                }
            };
            (frame.owner, object, pick)
        };
        assert!(exact_library_card(&self.state.objects, owner, object));
        match pick {
            AlternatingLibraryPick::Hand => {
                let event = self.move_object(card, Zone::Hand);
                if event.moved
                    && event.destination == Zone::Hand
                    && self.object_ref(card) == event.after
                    && !self.state.known_hand.contains(&event.after)
                {
                    self.state.known_hand.push(event.after);
                }
            }
            AlternatingLibraryPick::Bottom => {
                let previous_top = self.top_library_card(owner);
                self.place_library_bottom(owner, &[object]);
                if previous_top != self.top_library_card(owner) {
                    self.forget_library_top(owner);
                }
            }
        }
    }

    pub(super) fn library_selection_candidate(
        &self,
        frame: &LibrarySelection,
        object: ObjectRef,
    ) -> bool {
        if !frame.eligible.contains(&object)
            || frame.alternating.is_some()
                && !exact_library_card(&self.state.objects, frame.owner, object)
        {
            return false;
        }
        if let Some(colors) = frame.available_colors {
            let mut reachable = 1u32;
            for reference in frame.selected.iter().chain(std::iter::once(&object)) {
                let colors = self.object_view(reference.card).colors & colors;
                let mut next = 0u32;
                for used in 0u8..32 {
                    if reachable & (1u32 << used) == 0 {
                        continue;
                    }
                    for bit in [1u8, 2, 4, 8, 16] {
                        if colors & bit != 0 && used & bit == 0 {
                            next |= 1u32 << (used | bit);
                        }
                    }
                }
                reachable = next;
            }
            if reachable == 0 {
                return false;
            }
        }
        match frame.spec.count {
            LibrarySelectionCount::ManaValueBudget { maximum_total, .. } => {
                frame
                    .selected
                    .iter()
                    .copied()
                    .chain([object])
                    .map(|r| {
                        u128::from(self.state.objects[r.card as usize].card.fixed_mana_value())
                    })
                    .sum::<u128>()
                    <= u128::from(maximum_total)
            }
            _ => true,
        }
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
        if f.alternating.is_some() {
            self.commit_alternating_library_pick(card);
            if !self.advance_alternating_library_picks() {
                self.run_program();
            }
            return;
        }
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
        if f.top_land == Some(TopLandStage::HandThenLands) {
            self.continue_top_hand_then_lands(owner);
            return true;
        }
        if f.top_land == Some(TopLandStage::Hand) && f.selected.is_empty() {
            f.top_land = Some(TopLandStage::Graveyard);
            f.destination = LibraryPickDestination::Graveyard;
            f.spec.reveal = false;
            f.eligible.clone_from(&f.remaining);
            self.state.pending = Pending::EffectLibrarySelection { player: owner };
            return true;
        }
        if f.spec.remainder == LibraryRemainder::OptionalGraveyard
            && !f.remaining.is_empty()
            && !f.ordering
        {
            f.ordering = true;
            self.state.pending = Pending::EffectLibrarySelection { player: owner };
            return true;
        }
        if f.spec.remainder == LibraryRemainder::BottomAny && !f.remaining.is_empty() {
            f.ordering = true;
            self.state.pending = Pending::EffectLibrarySelection { player: owner };
            true
        } else {
            self.commit_library_selection(owner);
            false
        }
    }
    fn continue_top_hand_then_lands(&mut self, owner: Player) {
        let mut frame = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .take()
            .unwrap();
        if let Some(chosen) = frame.selected.pop() {
            assert_eq!(self.object_ref(chosen.card), chosen);
            assert_eq!(self.state.objects[chosen.card as usize].zone, Zone::Library);
            let events = self.move_objects_raw(&[chosen.card], Zone::Hand);
            self.finish_zone_events(&events, &[]);
        }
        // The first stage's candidate list is the entire original looked set.
        // A prevented hand move leaves that exact card eligible for this stage.
        frame.remaining = std::mem::take(&mut frame.eligible);
        frame.remaining.retain(|&card| {
            self.object_ref(card.card) == card
                && self.state.objects[card.card as usize].zone == Zone::Library
        });
        frame.eligible = frame
            .remaining
            .iter()
            .copied()
            .filter(|card| self.state.objects[card.card as usize].card.is_land())
            .collect();
        frame.top_land = None;
        frame.destination = LibraryPickDestination::BattlefieldTapped;
        frame.spec.filter = LibrarySelectionFilter::Kinds(2);
        frame.spec.count = LibrarySelectionCount::All;
        frame.spec.remainder = LibraryRemainder::Graveyard;
        frame.maximum = frame.eligible.len();
        self.state.resolving.as_mut().unwrap().selection = Some(frame);
        // Keep the same private choice boundary even when no looked land qualifies.
        self.state.pending = Pending::EffectLibrarySelection { player: owner };
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
        if f.top_land.is_none() || !f.selected.is_empty() {
            self.forget_library_top(owner);
        }
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
        let events = match f.destination {
            LibraryPickDestination::Hand => self.move_objects_raw(&ids, Zone::Hand),
            LibraryPickDestination::Graveyard => self.move_objects_raw(&ids, Zone::Graveyard),
            LibraryPickDestination::Exile | LibraryPickDestination::ExileCastThisTurn => {
                self.move_objects_raw(&ids, Zone::Exile)
            }
            _ => Vec::new(),
        };
        if f.destination == LibraryPickDestination::ExileCastThisTurn {
            for event in &events {
                if event.moved && event.destination == Zone::Exile {
                    self.grant_exile_play_permission(owner, event.after, false);
                    self.state
                        .exile_permissions
                        .iter_mut()
                        .find(|p| p.object == event.after && p.player == owner && !p.cast_only)
                        .unwrap()
                        .cast_only = true;
                }
            }
        }
        if f.destination == LibraryPickDestination::Hideaway {
            let events = self.move_objects_raw(&ids, Zone::Exile);
            let item = &self.state.resolving.as_ref().unwrap().item;
            let source = item.source.expect("hideaway has source").object;
            let origin = item.source_ability_origin;
            for event in &events {
                if event.moved && event.destination == Zone::Exile {
                    self.link_exile_to_ability(source, origin, event.after);
                    self.remember_face_down_exile(event.after, 1 << owner.index(), Some(source));
                }
            }
            self.finish_zone_events(&events, &[]);
        }
        if f.spec.reveal {
            for e in &events {
                if e.moved && e.destination == Zone::Hand {
                    self.state.known_hand.push(e.after);
                }
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
            LibraryPickDestination::Battlefield
                | LibraryPickDestination::BattlefieldTapped
                | LibraryPickDestination::BattlefieldTappedAttacking
        ) && !ids.is_empty()
        {
            f.entering = true;
            let tapped = f.destination != LibraryPickDestination::Battlefield;
            let attacking = f.destination == LibraryPickDestination::BattlefieldTappedAttacking;
            self.state.resolving.as_mut().unwrap().selection = Some(f);
            let moves = ids
                .into_iter()
                .map(|id| {
                    let mut movement = self.entry_move(id, Zone::Battlefield, tapped);
                    movement.choose_attacking_defender = attacking;
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
                LibraryPickDestination::Battlefield
                    | LibraryPickDestination::BattlefieldTapped
                    | LibraryPickDestination::BattlefieldTappedAttacking
            )
        {
            f.remaining.extend(f.selected.iter().copied().filter(|r| {
                self.object_ref(r.card) == *r
                    && self.state.objects[r.card as usize].zone == Zone::Library
            }));
        }
        match f.spec.remainder {
            LibraryRemainder::OptionalGraveyard => {
                let ids: Vec<_> = f.bottom.iter().map(|r| r.card).collect();
                let events = self.move_objects_raw(&ids, Zone::Graveyard);
                self.finish_zone_events(&events, &[]);
            }
            LibraryRemainder::Keep => (),
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
            LibraryRemainder::Hand => {
                let ids: Vec<_> = f.remaining.iter().map(|r| r.card).collect();
                let events = self.move_objects_raw(&ids, Zone::Hand);
                self.finish_zone_events(&events, &[]);
            }
            LibraryRemainder::BottomAny | LibraryRemainder::BottomRandom => {
                if f.spec.remainder == LibraryRemainder::BottomRandom {
                    f.bottom = std::mem::take(&mut f.remaining);
                    self.state.rng.shuffle(&mut f.bottom);
                }
                self.place_library_bottom(owner, &f.bottom);
            }
        }
    }
    pub(super) fn place_library_bottom(&mut self, owner: Player, order: &[ObjectRef]) {
        let library = &mut self.state.players[owner.index()].library;
        library.retain(|id| !order.iter().any(|r| r.card == *id));
        library.splice(0..0, order.iter().rev().map(|r| r.card));
    }
    pub(super) fn capture_selected_library_entry(&mut self, events: &[ZoneChange]) {
        let Some(frame) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.selection.as_ref())
            .filter(|frame| frame.entering && frame.completion != LibraryEntryCompletion::None)
        else {
            return;
        };
        let entered: Vec<_> = events
            .iter()
            .filter(|event| {
                event.moved
                    && event.destination == Zone::Battlefield
                    && frame.selected.contains(&event.before.reference)
            })
            .map(|event| event.after)
            .filter(|object| {
                frame.completion == LibraryEntryCompletion::MayTransform
                    || (self.current_kinds(object.card) & 16 != 0
                        && self.has_subtype(object.card, "Equipment"))
            })
            .collect();
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap()
            .entered
            .extend(entered);
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
        if accept && let Some(&recipient) = frame.entered.first() {
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
        let &object = frame.entered.first()?;
        (self.on_battlefield(object.card) && self.object_ref(object.card) == object).then(|| {
            EntryTransformChoiceView {
                object: self.object_view(object.card),
            }
        })
    }
    pub(super) fn finish_entered_library_selection(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let Some(frame) = resolution.selection.as_ref().filter(|frame| frame.entering) else {
            return false;
        };
        if frame.completion == LibraryEntryCompletion::MayTransform
            && frame
                .entered
                .first()
                .is_some_and(|&object| self.can_transform_permanent(object))
        {
            let resolution = self.state.resolving.as_mut().unwrap();
            let frame = resolution.selection.as_mut().unwrap();
            frame.entering = false;
            frame.transform_pending = true;
            self.state.pending = Pending::EffectOptional {
                player: frame.owner,
                skip_to: resolution.pc,
            };
            return true;
        }
        let resolution = self.state.resolving.as_mut().unwrap();
        let mut frame = resolution.selection.take().unwrap();
        if frame.destination == LibraryPickDestination::LandsTappedRestHand {
            self.library_selected_nonlands_to_hand(&frame);
        } else {
            let completion = frame.completion;
            let entered = std::mem::take(&mut frame.entered);
            self.commit_library_remainder(frame.owner, *frame);
            if let LibraryEntryCompletion::ReflexiveEquipment(ability) = completion
                && !entered.is_empty()
            {
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                self.queue_reflexive_objects(&item, ability, entered);
            }
        }
        false
    }
    pub(super) fn library_selection_view(&self, viewer: Player) -> Option<LibrarySelectionView> {
        let f = self.state.resolving.as_ref()?.selection.as_ref()?;
        if !self.shares_information(viewer, f.owner) && !f.public {
            return None;
        }
        let visible = |object: &&ObjectRef| {
            f.alternating.is_none() || exact_library_card(&self.state.objects, f.owner, **object)
        };
        Some(LibrarySelectionView {
            remaining: f
                .remaining
                .iter()
                .filter(visible)
                .map(|r| self.object_view(r.card))
                .collect(),
            candidates: f
                .remaining
                .iter()
                .copied()
                .filter(|&r| self.library_selection_candidate(f, r))
                .collect(),
            selected: f
                .selected
                .iter()
                .map(|r| self.object_view(r.card))
                .collect(),
            bottom: f
                .bottom
                .iter()
                .filter(visible)
                .map(|r| self.object_view(r.card))
                .collect(),
            minimum: f.minimum,
            maximum: f.maximum,
            ordering: f.ordering,
            random: f.random,
            available_colors: f.available_colors,
            reveal: f.spec.reveal,
            public: f.public,
            destination: match f.destination {
                _ if f
                    .alternating
                    .is_some_and(|picks| picks.next == AlternatingLibraryPick::Bottom) =>
                {
                    "bottom"
                }
                LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(_) => {
                    unreachable!("normalized before frame creation")
                }
                LibraryPickDestination::LandsTappedRestHand => "lands_tapped_rest_hand",
                LibraryPickDestination::Hand => "hand",
                LibraryPickDestination::Graveyard => "graveyard",
                LibraryPickDestination::Battlefield => "battlefield",
                LibraryPickDestination::BattlefieldTapped => "battlefield_tapped",
                LibraryPickDestination::BattlefieldTappedAttacking => {
                    "battlefield_tapped_attacking"
                }
                LibraryPickDestination::Top => "top",
                LibraryPickDestination::Hideaway => "hideaway",
                LibraryPickDestination::Exile => "exile",
                LibraryPickDestination::ExileCastThisTurn => "exile_cast_this_turn",
            },
            remainder: match f.spec.remainder {
                LibraryRemainder::Keep => "keep",
                LibraryRemainder::OptionalGraveyard => "optional_graveyard",
                LibraryRemainder::Graveyard => "graveyard",
                LibraryRemainder::Hand => "hand",
                LibraryRemainder::Shuffle => "shuffle",
                LibraryRemainder::BottomAny => "bottom_any",
                LibraryRemainder::BottomRandom => "bottom_random",
            },
        })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn exact_library_card(objects: &[Object], owner: Player, object: ObjectRef) -> bool {
    objects.get(object.card as usize).is_some_and(|card| {
        card.generation == object.generation
            && card.zone == Zone::Library
            && card.owner == owner
            && card.is_card()
    })
}
