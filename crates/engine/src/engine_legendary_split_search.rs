//! A qualified mixed-zone search with a private find and a public split.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Origin {
    Library(ObjectRef),
    Outside(u16),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Found {
    origin: Origin,
    card: Card,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Zones,
    Searching {
        library: bool,
        outside: bool,
    },
    Choosing,
    Admitting {
        hand: Vec<u16>,
        library: Vec<u16>,
        awaiting: Option<(u16, u64)>,
    },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LegendarySplitSearch {
    controller: Player,
    maximum: u16,
    opponent_choices: u16,
    phase: Phase,
    found: Vec<Found>,
    chosen: Vec<u16>,
}

impl LegendarySplitSearch {
    fn required(&self) -> usize {
        self.found.len().min(usize::from(self.opponent_choices))
    }

    fn qualifies(&self, card: Card) -> bool {
        card.is_creature()
            && card.is_legendary()
            && self.found.iter().all(|prior| {
                outside_names::different(prior.card, card).unwrap_or_else(|| {
                    !crate::card_names::same_name(prior.card.name(), card.name())
                })
            })
    }

    pub(super) fn owned_bytes(&self) -> usize {
        let slots = match &self.phase {
            Phase::Admitting { hand, library, .. } => hand.len() + library.len(),
            _ => 0,
        };
        std::mem::size_of::<Self>()
            + self.found.len() * std::mem::size_of::<Found>()
            + (self.chosen.len() + slots) * std::mem::size_of::<u16>()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LegendarySplitSearchView {
    pub controller: Player,
    pub choosing_zones: bool,
    pub revealed: bool,
    pub maximum: u16,
    pub required_for_hand: usize,
    pub can_finish: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<ObjectView>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub library_candidates: Vec<ObjectRef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outside_candidates: Vec<u16>,
    pub found: Vec<Card>,
    pub chosen_for_hand: Vec<u16>,
}

impl Game {
    pub(super) fn start_legendary_split_search(
        &mut self,
        controller: Player,
        maximum: u16,
        opponent_choices: u16,
    ) {
        self.state.resolving.as_mut().unwrap().legendary_search =
            Some(Box::new(LegendarySplitSearch {
                controller,
                maximum,
                opponent_choices,
                phase: Phase::Zones,
                found: Vec::new(),
                chosen: Vec::new(),
            }));
        self.state.pending = Pending::LegendarySplitSearch { player: controller };
    }

    fn legendary_search(&self) -> &LegendarySplitSearch {
        self.state
            .resolving
            .as_ref()
            .unwrap()
            .legendary_search
            .as_ref()
            .unwrap()
    }

    fn legendary_search_mut(&mut self) -> &mut LegendarySplitSearch {
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .legendary_search
            .as_mut()
            .unwrap()
    }

    fn legendary_candidates<'a>(
        &'a self,
        search: &'a LegendarySplitSearch,
    ) -> impl Iterator<Item = Found> + 'a {
        let player = search.controller;
        let room = search.found.len() < usize::from(search.maximum);
        let library = room && matches!(search.phase, Phase::Searching { library: true, .. });
        let outside = room && matches!(search.phase, Phase::Searching { outside: true, .. });
        let library = library.then(|| &self.state.players[player.index()].library);
        let outside = outside
            .then_some(self.state.outside_game.as_deref())
            .flatten();
        library
            .into_iter()
            .flatten()
            .filter_map(move |&id| {
                let object = &self.state.objects[id as usize];
                (object.is_card() && object.owner == player && search.qualifies(object.card)).then(
                    || Found {
                        origin: Origin::Library(self.object_ref(id)),
                        card: object.card,
                    },
                )
            })
            .chain(outside.into_iter().flat_map(move |state| {
                state.pools[player.index()]
                    .iter()
                    .copied()
                    .enumerate()
                    .filter_map(move |(index, card)| {
                        search.qualifies(card).then_some(Found {
                            origin: Origin::Outside(
                                u16::try_from(index).expect("outside pool bound"),
                            ),
                            card,
                        })
                    })
            }))
    }

    pub(super) fn visit_legendary_split_search(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let search = self.legendary_search();
        match search.phase {
            Phase::Zones => {
                emit(Action::ChooseLegendarySearchZones {
                    library: true,
                    outside: false,
                }) || (self.decision_controller(search.controller) == search.controller
                    && (emit(Action::ChooseLegendarySearchZones {
                        library: false,
                        outside: true,
                    }) || emit(Action::ChooseLegendarySearchZones {
                        library: true,
                        outside: true,
                    })))
            }
            Phase::Searching { .. } => {
                emit(Action::FinishSearch)
                    || self.legendary_candidates(search).any(|found| {
                        emit(match found.origin {
                            Origin::Library(object) => Action::ChooseSearch { card: object.card },
                            Origin::Outside(index) => Action::ChooseOutsideCard { index },
                        })
                    })
            }
            Phase::Choosing => {
                if search.chosen.len() == search.required() {
                    return emit(Action::FinishSearch);
                }
                (0..u16::try_from(search.found.len()).expect("search bound"))
                    .filter(|index| !search.chosen.contains(index))
                    .any(|index| emit(Action::ChooseLegendaryFound { index }))
            }
            Phase::Admitting { .. } => unreachable!("outside admission owns its decision"),
        }
    }

    pub(super) fn choose_legendary_search_zones(&mut self, library: bool, outside: bool) {
        self.legendary_search_mut().phase = Phase::Searching { library, outside };
    }

    pub(super) fn choose_legendary_library_card(&mut self, card: CardId) {
        let object = self.object_ref(card);
        let definition = self.state.objects[card as usize].card;
        self.legendary_search_mut().found.push(Found {
            origin: Origin::Library(object),
            card: definition,
        });
    }

    pub(super) fn choose_legendary_outside_card(&mut self, index: u16) {
        let owner = self.legendary_search().controller;
        let card =
            self.state.outside_game.as_ref().unwrap().pools[owner.index()][usize::from(index)];
        self.legendary_search_mut().found.push(Found {
            origin: Origin::Outside(index),
            card,
        });
    }

    pub(super) fn choose_legendary_found(&mut self, index: u16) {
        self.legendary_search_mut().chosen.push(index);
    }

    pub(super) fn finish_legendary_split_search(&mut self) {
        let mut search = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .legendary_search
            .take()
            .unwrap();
        let controller = search.controller;
        if let Phase::Searching { library, .. } = search.phase {
            if library {
                self.library_searched(controller);
            }
            for entry in &search.found {
                if let Origin::Outside(slot) = entry.origin {
                    self.reveal_outside_pool_card(controller, slot);
                }
            }
            if search.required() > 0 {
                search.phase = Phase::Choosing;
                self.state.resolving.as_mut().unwrap().legendary_search = Some(search);
                self.state.pending = Pending::LegendarySplitSearch {
                    player: controller.other(),
                };
                return;
            }
        }
        let mut hand = Vec::new();
        let mut library = Vec::new();
        let mut moves = Vec::new();
        for (index, entry) in search.found.iter().enumerate() {
            let chosen = search
                .chosen
                .contains(&u16::try_from(index).expect("search bound"));
            match entry.origin {
                Origin::Library(object) if chosen => {
                    assert_eq!(self.object_ref(object.card), object);
                    let mut movement = self.entry_move(object.card, Zone::Hand, false);
                    movement.reveal_hand = true;
                    moves.push(movement);
                }
                Origin::Outside(slot) if chosen => hand.push(slot),
                Origin::Outside(slot) => library.push(slot),
                Origin::Library(_) => {}
            }
        }
        // Taking a higher slot never invalidates the lower hand selections.
        hand.sort_unstable();
        library.sort_unstable();
        search.phase = Phase::Admitting {
            hand,
            library,
            awaiting: None,
        };
        self.state.resolving.as_mut().unwrap().legendary_search = Some(search);
        if !moves.is_empty() {
            self.stage_entries(moves, None);
        }
        self.run_program();
    }

    pub(super) fn advance_legendary_split_search(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(search) = resolution.legendary_search.as_mut() else {
            return false;
        };
        let Phase::Admitting {
            hand,
            library,
            awaiting,
        } = &mut search.phase
        else {
            return false;
        };
        let owner = search.controller;
        let pool_version = self
            .state
            .outside_game
            .as_ref()
            .map_or(0, |s| s.pool_versions[owner.index()]);
        if let Some((slot, prior)) = awaiting.take()
            && prior != pool_version
        {
            // A replacement can prevent admission. Shift only after an actual removal.
            for pending in library.iter_mut().filter(|pending| **pending > slot) {
                *pending -= 1;
            }
        }
        if let Some(slot) = hand.pop() {
            *awaiting = Some((slot, pool_version));
            let origin = resolution.item.id;
            self.admit_revealed_outside_to_hand(owner, origin, slot);
            return true;
        }
        let library = std::mem::take(library);
        resolution.legendary_search = None;
        for slot in library.into_iter().rev() {
            self.admit_revealed_outside_to_library(owner, slot);
        }
        self.forget_library_top(owner);
        self.shuffle_library_cards(owner);
        false
    }

    pub(super) fn legendary_split_view(&self, viewer: Player) -> Option<LegendarySplitSearchView> {
        let search = self.state.resolving.as_ref()?.legendary_search.as_ref()?;
        let revealed = matches!(search.phase, Phase::Choosing | Phase::Admitting { .. });
        if !revealed && !self.shares_information(viewer, search.controller) {
            return None;
        }
        let library = if matches!(search.phase, Phase::Searching { library: true, .. }) {
            self.state.players[search.controller.index()]
                .library
                .iter()
                .copied()
                .map(|id| self.object_view(id))
                .collect()
        } else {
            Vec::new()
        };
        let mut library_candidates = Vec::new();
        let mut outside_candidates = Vec::new();
        for found in self.legendary_candidates(search) {
            match found.origin {
                Origin::Library(object) => library_candidates.push(object),
                Origin::Outside(index) if viewer == search.controller => {
                    outside_candidates.push(index)
                }
                Origin::Outside(_) => {}
            }
        }
        Some(LegendarySplitSearchView {
            controller: search.controller,
            choosing_zones: matches!(search.phase, Phase::Zones),
            revealed,
            maximum: search.maximum,
            required_for_hand: search.required(),
            can_finish: matches!(search.phase, Phase::Searching { .. })
                || (search.phase == Phase::Choosing && search.chosen.len() == search.required()),
            library,
            library_candidates,
            outside_candidates,
            found: search.found.iter().map(|found| found.card).collect(),
            chosen_for_hand: search.chosen.clone(),
        })
    }
}
