//! Private qualified searches, owned selections, and exact successful entry results.
use super::*;
use crate::SearchDestination;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SearchResult {
    pub origin: Option<ObjectRef>,
    pub entered: Option<ObjectRef>,
    pub exiled: Option<ObjectRef>,
    pub put_in_hand: bool,
    pub capture_next_entry: bool,
    pub closed: bool,
    /// Further exact returns, for example the other half of a melded permanent.
    pub blink_companions: Vec<(ObjectRef, Option<ObjectRef>)>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SearchResult {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.blink_companions.len() * std::mem::size_of::<(ObjectRef, Option<ObjectRef>)>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SearchChoice {
    pub spec: SearchSpec,
    pub defer_shuffle: bool,
    pub entry_counters: &'static [(CounterKind, u64)],
    pub minimum: u64,
    pub different_names: bool,
    pub distinct_colors: u8,
    pub paired_qualities: Option<&'static [crate::SearchFilter; 2]>,
    pub selected: Vec<CardId>,
    /// Revealed cards assigned by the active distribution chooser.
    pub distributed: Option<Vec<CardId>>,
    pub destination_pending: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SearchChoice {
    pub(super) fn qualifies_for_open_role(
        &self,
        game: &Game,
        player: Player,
        candidate: CardId,
    ) -> bool {
        let Some(roles) = self.paired_qualities else {
            return true;
        };
        let Some(&first) = self.selected.first() else {
            return true;
        };
        let matches = |role: usize, id| game.card_search_filter_matches(roles[role], player, id);
        (matches(0, first) && matches(1, candidate)) || (matches(1, first) && matches(0, candidate))
    }

    pub(super) fn colors_allow(&self, game: &Game, candidate: CardId) -> bool {
        if self.distinct_colors == 0 {
            return true;
        }
        // A bit in reachable denotes a set of occupied WUBRG roles. There are
        // only32 such sets; no greedy commitment of a multicolor card is made.
        let mut reachable = 1u32;
        for &card in self.selected.iter().chain(std::iter::once(&candidate)) {
            let available = game.state.objects[card as usize].card.colors() & self.distinct_colors;
            let mut next = 0u32;
            for occupied in 0u8..32 {
                if reachable & (1u32 << occupied) == 0 {
                    continue;
                }
                for color in 0..5 {
                    let role = 1u8 << color;
                    if available & role != 0 && occupied & role == 0 {
                        next |= 1u32 << (occupied | role);
                    }
                }
            }
            reachable = next;
            if reachable == 0 {
                return false;
            }
        }
        true
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn resolution_target_controller(&self) -> Option<Player> {
        let resolution = self.state.resolving.as_ref()?;
        resolution
            .target_card_facts
            .as_ref()?
            .iter()
            .find(|fact| fact.group == resolution.target_group)
            .map(|fact| fact.controller)
    }

    pub(super) fn card_search_filter_matches(
        &self,
        filter: crate::cards::SearchFilter,
        player: Player,
        id: CardId,
    ) -> bool {
        use crate::cards::SearchFilter;
        match filter {
            SearchFilter::RoomWithoutSharedName => {
                let card = self.state.objects[id as usize].card;
                card.has_subtype("Room")
                    && !self.state.objects.iter().enumerate().any(|(id, object)| {
                        let id = id as CardId;
                        object.zone == Zone::Battlefield
                            && object.controller == player
                            && self.has_subtype(id, "Room")
                            && crate::card_names::same_name(card.name(), self.object_name(id))
                    })
            }
            SearchFilter::AnyOf(filters) => filters
                .iter()
                .any(|&f| self.card_search_filter_matches(f, player, id)),
            SearchFilter::AllOf(filters) => filters
                .iter()
                .all(|&f| self.card_search_filter_matches(f, player, id)),
            SearchFilter::Subtype(subtype) => self.has_subtype(id, subtype),
            SearchFilter::NonSubtype(subtype) => !self.has_subtype(id, subtype),
            _ => filter.matches(self.state.objects[id as usize].card),
        }
    }

    pub(super) fn choose_search(&mut self, card: CardId) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        choice.selected.push(card);
        // Selections remain private and in the library until FinishSearch. Even
        // a maximum-sized selection has an explicit commit boundary for replay.
    }

    pub(super) fn choose_search_distribution(&mut self, chooser: Player, card: CardId) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let choice = resolution.search.as_mut().unwrap();
        let (player, count) = match choice.spec.destination {
            SearchDestination::BattlefieldThenHand { battlefield } => (chooser, battlefield),
            SearchDestination::BattlefieldAndOpponentChosenGraveyard { graveyard } => {
                (chooser.other(), graveyard)
            }
            _ => unreachable!("validated split-search action"),
        };
        let chosen = choice.distributed.as_mut().unwrap();
        chosen.push(card);
        if chosen.len() == usize::from(count) {
            self.commit_search(player);
        }
    }

    pub(super) fn finish_search(&mut self, player: Player) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        if choice.spec.destination == SearchDestination::HandOrGraveyard
            && !choice.selected.is_empty()
        {
            choice.destination_pending = true;
            return;
        }
        match choice.spec.destination {
            SearchDestination::BattlefieldThenHand { battlefield } => {
                if battlefield == 0 {
                    choice.distributed = Some(Vec::new());
                } else if choice.selected.len() > usize::from(battlefield) {
                    choice.distributed = Some(Vec::new());
                    return;
                }
            }
            SearchDestination::BattlefieldAndOpponentChosenGraveyard { graveyard } => {
                assert!(
                    choice.spec.reveal,
                    "opponent assigns revealed search results"
                );
                if graveyard == 0 {
                    choice.distributed = Some(Vec::new());
                } else if choice.selected.len() > usize::from(graveyard) {
                    // The selected identities become public when the opponent
                    // takes over. No other library cards are disclosed.
                    choice.distributed = Some(Vec::new());
                    let revealed: Vec<_> = choice
                        .selected
                        .iter()
                        .map(|&id| self.state.objects[id as usize].card)
                        .collect();
                    for card in revealed {
                        self.record_zone_reveal(card, player, Zone::Library);
                    }
                    self.state.pending = Pending::EffectSearch {
                        player: player.other(),
                    };
                    return;
                }
            }
            _ => {}
        }
        self.commit_search(player);
    }

    pub(super) fn choose_search_destination(
        &mut self,
        player: Player,
        destination: SearchDestination,
    ) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap();
        assert!(
            choice.destination_pending
                && matches!(
                    destination,
                    SearchDestination::Hand | SearchDestination::Graveyard
                )
        );
        choice.destination_pending = false;
        choice.spec.destination = destination;
        self.commit_search(player);
    }
    pub(super) fn library_searched(&mut self, player: Player) {
        // Freeze observers before the found cards move, so a creature fetched
        // by this search does not see the search that found it.
        let sources = self.indexed_trigger_sources(&self.state.draw_sources);
        for (source, grant) in sources {
            for (index, ability) in source.trigger_templates_from(grant.as_ref()) {
                if let TriggerCondition::LibrarySearched { relation } = ability.condition.event()
                    && match relation {
                        CastRelation::You => source.controller == player,
                        CastRelation::Opponent => source.controller != player,
                        CastRelation::Any => true,
                    }
                {
                    self.queue_trigger_from(&source, index, grant.map(Box::new));
                }
            }
        }
    }
    fn commit_search(&mut self, player: Player) {
        self.library_searched(player);
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .take()
            .unwrap();
        let origin = choice.selected.first().map(|&id| self.object_ref(id));
        if choice.defer_shuffle {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .deferred_search_shuffle = true;
        }
        if let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search_result
            .as_mut()
        {
            frame.origin = origin;
            frame.exiled = None;
            frame.closed = false;
        }
        if choice.spec.destination == SearchDestination::Top {
            assert!(
                !choice.defer_shuffle,
                "a top search must shuffle before setting the top"
            );
            // Searching does not change the found card's zone or incarnation.
            // Reveal and shuffle are one committed action; only the final known
            // top is public, never the rest of the private search/selection.
            if self.state.players[player.index()].library.len() > 1 {
                self.forget_library_top(player);
            }
            self.shuffle_library_cards(player);
            let library = &mut self.state.players[player.index()].library;
            for &id in &choice.selected {
                let position = library.iter().position(|&card| card == id).unwrap();
                library.remove(position);
                library.push(id);
            }
            if choice.spec.reveal
                && let Some(&id) = choice.selected.last()
            {
                self.remember_library_top(player, self.object_ref(id));
            }
            self.run_program();
            return;
        }
        // A private search does not disclose whether the previously known top
        // was chosen or shuffled away. A singleton library is still deducible.
        if (choice.spec.destination == SearchDestination::ExileFaceDown
            || choice.spec.destination == SearchDestination::Hand && !choice.spec.reveal)
            && self.state.players[player.index()].library.len() > 1
        {
            self.forget_library_top(player);
        }
        if choice.spec.destination == SearchDestination::ExileFaceDown {
            let events = self.move_objects_raw(&choice.selected, Zone::Exile);
            for event in &events {
                if event.moved && event.destination == Zone::Exile {
                    self.remember_face_down_exile(event.after, 1 << player.index(), None);
                }
            }
            self.capture_search_result_entry(&events);
            self.finish_zone_events(&events, &[]);
            if !choice.defer_shuffle {
                self.shuffle_library_cards(player);
            }
            self.run_program();
            return;
        }
        let already_revealed = matches!(
            choice.spec.destination,
            SearchDestination::BattlefieldAndOpponentChosenGraveyard { graveyard }
                if graveyard != 0 && choice.selected.len() > usize::from(graveyard)
        );
        if choice.spec.reveal && !already_revealed {
            for &id in &choice.selected {
                let card = self.state.objects[id as usize].card;
                self.record_zone_reveal(card, player, Zone::Library);
            }
        }
        let moves = choice
            .selected
            .iter()
            .map(|&id| {
                let destination = match choice.spec.destination {
                    SearchDestination::Top => {
                        unreachable!("top search committed before entry moves")
                    }
                    SearchDestination::HandOrGraveyard => {
                        unreachable!("destination chosen before movement")
                    }
                    SearchDestination::ExileFaceDown => {
                        unreachable!("face-down exile committed before entry moves")
                    }
                    SearchDestination::LinkedExile | SearchDestination::ExileFreeThisTurn => {
                        Zone::Exile
                    }
                    SearchDestination::Hand => Zone::Hand,
                    SearchDestination::BeheldCreatureType => {
                        let kind = self
                            .state
                            .resolving
                            .as_ref()
                            .unwrap()
                            .item
                            .cast
                            .and_then(|c| c.beheld_type);
                        if kind.is_some_and(|kind| {
                            self.state.objects[id as usize]
                                .card
                                .has_subtype(kind.name())
                        }) {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    }
                    SearchDestination::Graveyard => Zone::Graveyard,
                    SearchDestination::Battlefield => Zone::Battlefield,
                    SearchDestination::BattlefieldIfLandElseHand => {
                        if self.current_kinds(id) & 2 != 0 {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    }
                    SearchDestination::BattlefieldIfManaValueAtMostElseHand(maximum) => {
                        if self.state.objects[id as usize].card.fixed_mana_value() <= maximum {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    }
                    SearchDestination::BattlefieldAndOpponentChosenGraveyard { .. } => {
                        if choice
                            .distributed
                            .as_ref()
                            .is_none_or(|assigned| assigned.contains(&id))
                        {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    }
                    SearchDestination::BattlefieldThenHand { .. } => {
                        if choice
                            .distributed
                            .as_ref()
                            .is_none_or(|selected| selected.contains(&id))
                        {
                            Zone::Battlefield
                        } else {
                            Zone::Hand
                        }
                    }
                };
                let mut movement = self.entry_move(
                    id,
                    destination,
                    destination == Zone::Battlefield && choice.spec.tapped,
                );
                movement.effect_counters = choice.entry_counters;
                movement.reveal_hand = choice.spec.reveal;
                movement.public_origin = choice.spec.reveal;
                if choice.spec.destination == SearchDestination::ExileFreeThisTurn {
                    movement.exile_free_until_end = Some(player);
                }
                movement
            })
            .collect();
        self.stage_entries(moves, (!choice.defer_shuffle).then_some(player));
        if choice.spec.destination == SearchDestination::LinkedExile {
            self.remember_linked_exile(&choice.selected);
        }
        self.run_program();
    }

    pub(super) fn capture_search_result_entry(&mut self, events: &[ZoneChange]) {
        let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .and_then(|r| r.search_result.as_mut())
        else {
            return;
        };
        if frame.closed {
            return;
        }
        frame.closed = true;
        let movement = events
            .iter()
            .find(|event| Some(event.before.reference) == frame.origin && event.moved);
        frame.entered = movement
            .filter(|event| event.destination == Zone::Battlefield)
            .map(|event| event.after);
        frame.exiled = movement
            .filter(|event| event.destination == Zone::Exile)
            .map(|event| event.after);
        frame.put_in_hand = movement.is_some_and(|event| event.destination == Zone::Hand);
        for (origin, entered) in &mut frame.blink_companions {
            *entered = events.iter().find_map(|event| {
                (event.before.reference == *origin
                    && event.moved
                    && event.destination == Zone::Battlefield)
                    .then_some(event.after)
            });
        }
    }

    pub(super) fn exiled_search_result(&self) -> Option<ObjectRef> {
        self.state
            .resolving
            .as_ref()?
            .search_result
            .as_ref()?
            .exiled
            .filter(|object| {
                self.object_ref(object.card) == *object
                    && self.state.objects[object.card as usize].zone == Zone::Exile
            })
    }
}

/// Public saved recipients: independent of the land/source after destruction.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OptionalSearchSequence {
    pub players: [Player; 2],
    pub next: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub land_controller_only: bool,
    #[serde(skip)]
    pub tapped: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn destroy_land_and_search(&mut self, item: &SpellView, land_controller_only: bool) {
        let Some(bound) = item
            .targets
            .first()
            .copied()
            .filter(|bound| self.same_target_object(*bound))
        else {
            return;
        };
        let Target::Card(id) = bound.target else {
            return;
        };
        let affected = self.state.objects[id as usize].controller;
        self.state.resolving.as_mut().unwrap().optional_searches =
            Some(Box::new(OptionalSearchSequence {
                players: [affected, item.controller],
                next: 0,
                land_controller_only,
                tapped: false,
            }));
        if !self.effect_destruction_prevented(id) {
            self.move_target_batch(&[bound], Zone::Graveyard);
        }
    }
    pub(super) fn nonbasic_land_choice(&self, object: ObjectRef) -> bool {
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|land| {
                land.generation == object.generation
                    && land.zone == Zone::Battlefield
                    && self.current_kinds(object.card) & 2 != 0
                    && !land.card.is_basic_land()
            })
    }

    pub(super) fn finish_destroy_nonbasic_lands_for_each_player(&mut self) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let lands: Vec<_> = choice
            .selected
            .into_iter()
            .flatten()
            .filter(|&object| self.nonbasic_land_choice(object))
            .collect();
        if lands.is_empty() {
            return;
        }
        let events = self.destroy_objects_outcome(lands);
        let active = self.state.active;
        let mut affected = [active, active.other()].into_iter().filter(|&player| {
            events
                .iter()
                .any(|event| event.moved && event.before.controller == player)
        });
        if let Some(first) = affected.next() {
            let second = affected.next();
            self.state.resolving.as_mut().unwrap().optional_searches =
                Some(Box::new(OptionalSearchSequence {
                    players: [first, second.unwrap_or(first)],
                    next: 0,
                    land_controller_only: second.is_none(),
                    tapped: true,
                }));
        }
    }

    pub(super) fn advance_optional_searches(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(sequence) = resolution.optional_searches.as_ref() else {
            return false;
        };
        if resolution.search.is_some() {
            return false;
        }
        let participants = if sequence.land_controller_only {
            1
        } else {
            sequence.players.len()
        };
        if let Some(&player) = sequence.players[..participants].get(sequence.next) {
            self.state.pending = Pending::OptionalSearch(player);
            true
        } else {
            resolution.optional_searches = None;
            false
        }
    }
    pub(super) fn choose_optional_search(&mut self, player: Player, accept: bool) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let sequence = resolution.optional_searches.as_mut().unwrap();
        sequence.next += 1;
        let tapped = sequence.tapped;
        if accept {
            resolution.search = Some(Box::new(SearchChoice {
                defer_shuffle: false,
                spec: SearchSpec {
                    filter: crate::SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped,
                    reveal: false,
                },
                entry_counters: &[],
                minimum: 0,
                different_names: false,
                distinct_colors: 0,
                paired_qualities: None,
                selected: Vec::new(),
                distributed: None,
                destination_pending: false,
            }));
            self.state.pending = Pending::EffectSearch { player };
        } else {
            self.run_program();
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn captured_entry_result_has_subtype(&self, subtypes: &[&str]) -> bool {
        self.state
            .resolving
            .as_ref()
            .and_then(|resolution| resolution.search_result.as_ref())
            .is_some_and(|frame| {
                frame
                    .entered
                    .into_iter()
                    .chain(
                        frame
                            .blink_companions
                            .iter()
                            .filter_map(|(_, entered)| *entered),
                    )
                    .any(|object| {
                        self.on_battlefield(object.card)
                            && self.object_ref(object.card) == object
                            && subtypes
                                .iter()
                                .any(|subtype| self.has_subtype(object.card, subtype))
                    })
            })
    }

    pub(super) fn captured_entry_result(&self) -> Option<ObjectRef> {
        self.state
            .resolving
            .as_ref()?
            .search_result
            .as_ref()?
            .entered
            .filter(|object| {
                self.on_battlefield(object.card) && self.object_ref(object.card) == *object
            })
    }
}
