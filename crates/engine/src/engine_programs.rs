//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;
use crate::cards::{CastFilter, CastTarget, StaticAbilityFragment};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TargetCardFacts {
    pub group: u16,
    pub controller: Player,
    pub mana_value: u64,
    pub attacking: bool,
    pub legendary: bool,
    pub kinds: u16,
    pub power: i128,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OptionalWheel {
    pub first: Player,
    pub choices: [Option<bool>; 2],
    pub amount: u16,
    pub next: u8,
    pub running: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub chosen_card_name: Option<crate::CardName>,
    pub chosen_creature_type: Option<crate::cards::CreatureType>,
    pub target_counter_removal: Option<Box<TargetCounterRemovalView>>,
    pub linked_copy: Option<Box<linked_copies::LinkedCopyCapture>>,
    pub cascade: Option<Box<cascade::CascadeView>>,
    #[allow(clippy::box_collection)]
    pub target_card_facts: Option<Box<Vec<TargetCardFacts>>>,
    #[allow(clippy::box_collection)]
    pub immediate_blink_return: Option<Box<Vec<ReturnLinkView>>>,
    pub connive: Option<Box<super::connive::ConniveSequence>>,
    pub proliferate: Option<Box<super::proliferate::ProliferateSequence>>,
    pub explore: Option<Box<ExploreChoiceView>>,
    pub evidence: Option<Box<evidence::EvidenceSelection>>,
    pub item: SpellView,
    pub mana_payment: Option<Box<resolution_mana::ResolutionPayment>>,
    pub forage: Option<Box<forage::ForageSelection>>,
    pub target_library: Option<Box<target_library::TargetLibraryChoice>>,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub target_group: u16,
    pub moved_count: Option<u64>,
    pub remembered_draws: Option<u64>,
    pub remembered_creatures: Vec<ObjectRef>,
    pub sacrificed_power: Option<Box<i128>>,
    pub discard_after: Option<discard_events::DiscardAfter>,
    pub optional_wheel: Option<OptionalWheel>,
    pub discards: [usize; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub milled: Vec<ObjectRef>,
    pub milled_selection: Option<Box<super::milled_selection::MilledSelection>>,
    pub library: Option<LibraryChoice>,
    pub selection: Option<Box<super::library_selection::LibrarySelection>>,
    pub return_attachment: Option<Box<super::return_attachment::ReturnAttachment>>,
    pub search: Option<Box<super::search::SearchChoice>>,
    pub search_result: Option<Box<super::search::SearchResult>>,
    pub optional_searches: Option<Box<super::search::OptionalSearchSequence>>,
    pub choice: Option<Box<super::effect_choices::EffectChoice>>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibraryChoice {
    pub surveil: bool,
    pub return_within_life_gained: bool,
    pub return_noncreature_nonland: bool,
    pub remaining: Vec<CardId>,
    pub top: Vec<CardId>,
    pub bottom: Vec<CardId>,
    pub graveyard: Vec<CardId>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn entering_subject(&self, item: &SpellView) -> Option<CardId> {
        item.entering_object
            .filter(|object| {
                self.on_battlefield(object.card) && self.object_ref(object.card) == *object
            })
            .map(|object| object.card)
    }

    fn subject_condition(
        &self,
        item: &SpellView,
        subject: Subject,
        filter: crate::cards::CreatureFilter,
    ) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        let id = match subject {
            Subject::Entering => self.entering_subject(item),
            Subject::Source => self
                .on_battlefield(source.id)
                .then_some(source.id)
                .filter(|&id| self.object_ref(id) == source.object),
            Subject::Attached => self.attached_subject(item),
            Subject::Targets => match item.targets.as_slice() {
                [bound] if self.same_target_object(*bound) => match bound.target {
                    Target::Card(id) => Some(id),
                    _ => None,
                },
                _ => None,
            },
        };
        id.is_some_and(|id| self.creature_matches(filter, item.controller, source.object, id))
    }

    pub(super) fn arrange_library(
        &mut self,
        player: Player,
        card: CardId,
        destination: LibraryDestination,
    ) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .library
            .as_mut()
            .unwrap();
        let index = choice.remaining.iter().position(|&id| id == card).unwrap();
        choice.remaining.remove(index);
        match destination {
            LibraryDestination::Top => choice.top.push(card),
            LibraryDestination::Bottom => choice.bottom.push(card),
            LibraryDestination::Graveyard => choice.graveyard.push(card),
            LibraryDestination::Shuffle
            | LibraryDestination::SecondFromTop
            | LibraryDestination::NthFromTop(_) => {
                unreachable!("destination is not an arrangement action")
            }
        }
        if !choice.remaining.is_empty() {
            return;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .library
            .take()
            .unwrap();
        self.forget_library_top(player);
        // No zone changes or observable partial rearrangement occur while choosing.
        // Surveilled cards enter the graveyard as one simultaneous event.
        // Freeze replacement plans while retained cards are still visible, then
        // reserve their specified positions before committing any shuffles.
        let events = self.move_objects_raw_with_library_positions(
            &choice.graveyard,
            Zone::Graveyard,
            Some((player, &choice.top, &choice.bottom)),
        );
        let mut return_cards = Vec::new();
        for event in &events {
            let card = event.before.reference.card;
            if event.moved
                && event.destination == Zone::Graveyard
                && self.object_ref(card) == event.after
                && self.state.objects[card as usize].zone == Zone::Graveyard
                && ((choice.return_within_life_gained
                    && u128::from(self.state.objects[card as usize].card.fixed_mana_value())
                        <= self.state.players[player.index()].life_gained_this_turn)
                    || (choice.return_noncreature_nonland
                        && self.current_kinds(card) & (1 | 2) == 0))
            {
                return_cards.push(event.after);
            }
        }
        let library = &mut self.state.players[player.index()].library;
        library.splice(0..0, choice.bottom.iter().rev().copied());
        library.extend(choice.top.iter().rev().copied());
        self.refresh_controllers();
        self.finish_zone_events(&events, &[]);
        self.scry_or_surveil_event(player);
        for reference in return_cards {
            if self.object_ref(reference.card) == reference
                && self.state.objects[reference.card as usize].zone == Zone::Graveyard
            {
                self.move_object(reference.card, Zone::Hand);
            }
        }
        self.run_program();
    }

    fn scry_or_surveil_event(&mut self, player: Player) {
        self.state.players[player.index()].scried_or_surveilled_this_turn = true;
        let sources = self.indexed_trigger_sources(&self.state.scry_sources);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::ScryOrSurveil
                    && source.controller == player
                {
                    self.queue_trigger_from(&source, index, grant.map(Box::new));
                }
            }
        }
    }

    pub(super) fn next_stack_id(&mut self) -> u64 {
        let id = self.state.next_stack_id;
        self.state.next_stack_id = id.checked_add(1).capacity("stack identity exhausted");
        id
    }

    pub(super) fn lose_life(&mut self, player: Player, amount: i128) {
        self.lose_life_unobserved(player, amount);
        self.life_lost_event(player, amount);
    }

    /// Pay a life cost, as distinct from an instruction that causes life loss.
    pub(super) fn pay_life(&mut self, player: Player, amount: i128) {
        if amount <= 0 {
            return;
        }
        let frozen = self.take_frozen_mana_life_payment();
        if let Some(None) = frozen {
            return;
        }
        if let Some(Some(receipt)) = &frozen {
            if receipt.player != player || receipt.amount != amount {
                self.fail_frozen_mana_replay();
                return;
            }
            if receipt.exiled.is_none() {
                self.lose_life(player, amount);
                return;
            }
        }
        let applies = self.state.static_sources.iter().copied().any(|id| {
            let source = &self.state.objects[id as usize];
            source.zone == Zone::Battlefield
                && source.controller == player
                && self.printed_abilities_available(id)
                && source.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::ReplaceLifePaymentWithLibraryExile
                        )
                    })
                })
        });
        if applies || frozen.is_some() {
            let excluded = self
                .state
                .entry_batch
                .as_ref()
                .map(|batch| {
                    batch
                        .moves
                        .iter()
                        .filter(|movement| {
                            !movement.skipped
                                && movement.destination == Zone::Battlefield
                                && movement.before.zone == Zone::Library
                        })
                        .map(|movement| movement.before.reference)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let count = usize::try_from(amount).unwrap_or(usize::MAX);
            let cards = self.state.players[player.index()]
                .library
                .iter()
                .rev()
                .copied()
                .filter(|&id| {
                    self.state.objects[id as usize].is_card()
                        && !excluded.contains(&self.object_ref(id))
                })
                .take(count)
                .collect::<Vec<_>>();
            let references = cards
                .iter()
                .map(|&id| self.object_ref(id))
                .collect::<Vec<_>>();
            if let Some(Some(receipt)) = &frozen
                && receipt.exiled.as_ref() != Some(&references)
            {
                self.fail_frozen_mana_replay();
                return;
            }
            if cards.len() == count {
                self.record_cast_mana_life_payment(player, amount, Some(references));
                let events = self.move_objects_raw(&cards, Zone::Exile);
                self.finish_zone_events(&events, &[]);
                return;
            }
        }
        self.record_cast_mana_life_payment(player, amount, None);
        self.lose_life(player, amount);
    }

    pub(super) fn lose_life_unobserved(&mut self, player: Player, amount: i128) {
        if amount <= 0 {
            return;
        }
        let state = &mut self.state.players[player.index()];
        state.life = state
            .life
            .checked_sub(amount)
            .capacity("life representation exhausted");
        state.life_lost_this_turn = state
            .life_lost_this_turn
            .checked_add(amount as u128)
            .capacity("life lost this turn representation exhausted");
    }

    pub(super) fn life_lost_event(&mut self, player: Player, amount: i128) {
        if amount <= 0 {
            return;
        }
        let active = self.state.active;
        let designation = &mut self.state.players[active.index()];
        if player != active
            && designation.speed > 0
            && designation.speed < 4
            && !designation.speed_triggered_this_turn
        {
            designation.speed_triggered_this_turn = true;
            self.queue_inherent_trigger(SourceLessRule::Speed, active);
        }
        let sources: Vec<_> = self
            .state
            .life_sources
            .iter()
            .filter(|&&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|a| matches!(a.condition.event(), TriggerCondition::LifeLost { .. }))
            })
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::LifeLost { relation } = ability.condition.event() {
                    let related = match relation {
                        CastRelation::You => source.controller == player,
                        CastRelation::Opponent => source.controller != player,
                        CastRelation::Any => true,
                    };
                    if related && self.queue_trigger(&source, index) {
                        self.state.pending_triggers.last_mut().unwrap().life_event =
                            Some(LifeTriggerView { player, amount });
                    }
                }
            }
        }
    }

    pub(super) fn has_counter_batch_observers(&self) -> bool {
        self.state.counter_sources.iter().any(|&id| {
            self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|a| {
                    matches!(
                        a.condition.event(),
                        TriggerCondition::CounterBatchPlacedByYou { .. }
                            | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                    )
                })
        })
    }

    /// Dispatch once per observing ability after every recipient of a simultaneous
    /// placement has committed. The iterator borrows transient amounts; no event
    /// identity or latch is retained in owned game state.
    pub(super) fn counter_batch_added<'a>(
        &mut self,
        placements: impl Iterator<Item = (ObjectRef, &'a Counters, Player)> + Clone,
    ) {
        let sources: Vec<_> = self
            .state
            .counter_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.state.objects[id as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|a| {
                        matches!(
                            a.condition.event(),
                            TriggerCondition::CounterBatchPlacedByYou { .. }
                                | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                        )
                    })
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                match ability.condition.event() {
                    TriggerCondition::CounterPlacedByYouOnControlledSubtypes { subtypes } => {
                        let positive = placements.clone().filter(|(_, amount, _)| amount.has_any());
                        for (position, (object, _, placed_by)) in positive.clone().enumerate() {
                            if !positive
                                .clone()
                                .take(position)
                                .any(|(previous, _, player)| {
                                    previous == object && player == placed_by
                                })
                                && placed_by == source.controller
                                && self.on_battlefield(object.card)
                                && self.object_ref(object.card) == object
                                && self.state.objects[object.card as usize].controller
                                    == source.controller
                                && subtypes
                                    .iter()
                                    .any(|subtype| self.has_subtype(object.card, subtype))
                            {
                                self.queue_trigger(&source, index);
                            }
                        }
                    }
                    TriggerCondition::CounterBatchPlacedByYou { kind, filter } => {
                        let qualifies = placements.clone().any(|(object, amount, placed_by)| {
                            amount.get(kind) > 0
                                && placed_by == source.controller
                                && self.on_battlefield(object.card)
                                && self.object_ref(object.card) == object
                                && Self::count_object_matches(
                                    filter,
                                    source.reference,
                                    source.controller,
                                    object.card,
                                    &self.state.objects[object.card as usize],
                                    &|id| self.granted_subtypes(id),
                                    &|id, o| self.object_kinds(id, o),
                                )
                        });
                        if qualifies {
                            self.queue_trigger(&source, index);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(super) fn self_counters_added(
        &mut self,
        object: ObjectRef,
        kind: CounterKind,
        amount: u64,
        placed_by: Player,
    ) {
        if amount == 0
            || !self.on_battlefield(object.card)
            || self.object_ref(object.card) != object
        {
            return;
        }
        self.record_counter_placement(object, placed_by, kind == CounterKind::PlusOne);
        if self.state.objects[object.card as usize].card.triggers().iter().any(|a| {
            matches!(a.condition.event(), TriggerCondition::CounterThresholdCrossed { kind: expected, .. } if expected == kind)
        }) {
            let after = self.counter_count(object.card, kind);
            let before = after.checked_sub(amount).expect("committed placement amount");
            let source = self.object_view(object.card);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::CounterThresholdCrossed {
                    kind: expected,
                    threshold,
                } = ability.condition.event()
                    && kind == expected
                    && before < threshold
                    && threshold <= after
                {
                    self.queue_trigger(&source, index);
                }
            }
        }
        self.queue_state_counter_triggers();
        if !self.state.counter_sources.is_empty() {
            let recipient = self.object_view(object.card);
            let observers: Vec<_> = self
                .state
                .counter_sources
                .iter()
                .copied()
                .map(|id| self.object_view(id))
                .collect();
            for source in observers {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    let qualifies = match ability.condition.event() {
                        TriggerCondition::OtherCreatureCountersAdded {
                            kind: expected,
                            excluded_subtype,
                        } => {
                            expected == kind
                                && source.reference != object
                                && recipient.controller == source.controller
                                && recipient.is_creature()
                                && !recipient.has_subtype(excluded_subtype)
                        }
                        TriggerCondition::OtherCreatureCounterPlacedByYou { kind: expected } => {
                            expected == kind
                                && source.reference != object
                                && placed_by == source.controller
                                && recipient.is_creature()
                        }
                        TriggerCondition::CounterPlacedByYou {
                            kind: expected,
                            filter,
                        } => {
                            expected == kind
                                && placed_by == source.controller
                                && Self::count_object_matches(
                                    filter,
                                    source.reference,
                                    source.controller,
                                    object.card,
                                    &self.state.objects[object.card as usize],
                                    &|id| self.granted_subtypes(id),
                                    &|id, o| self.object_kinds(id, o),
                                )
                        }
                        _ => false,
                    };
                    if qualifies {
                        self.queue_trigger(&source, index);
                    }
                }
            }
        }
        if !self.state.objects[object.card as usize].card.triggers().iter().any(|a| matches!(a.condition.event(), TriggerCondition::SelfCountersAdded { kind: expected, .. } if expected == kind)) { return; }
        let source = self.object_view(object.card);
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if let TriggerCondition::SelfCountersAdded {
                kind: expected,
                controller_only,
            } = ability.condition.event()
                && kind == expected
                && (!controller_only || source.controller == placed_by)
            {
                self.queue_trigger(&source, index);
            }
        }
    }

    pub(super) fn gain_life(&mut self, player: Player, amount: i128) {
        if amount <= 0 {
            return;
        }
        // These additive replacements commute; each available source applies once.
        // Prohibition takes precedence and creates no life-gain event (CR101.2).
        let query = self.characteristic_query();
        let mut additional = 0i128;
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    match fragment {
                        StaticAbilityFragment::CantGainLife
                            if query.continuous_fragment_available(id, |f| {
                                matches!(f, StaticAbilityFragment::CantGainLife)
                            }) =>
                        {
                            return;
                        }
                        StaticAbilityFragment::LifeGainAdditional(n)
                            if source.controller == player
                                && query.continuous_fragment_available(id, |f| {
                                    matches!(f, StaticAbilityFragment::LifeGainAdditional(_))
                                }) =>
                        {
                            additional = additional
                                .checked_add(i128::from(*n))
                                .capacity("life replacement exhausted");
                        }
                        _ => {}
                    }
                }
            }
        }
        let amount = amount
            .checked_add(additional)
            .capacity("life replacement exhausted");
        let state = &mut self.state.players[player.index()];
        state.life = state
            .life
            .checked_add(amount)
            .capacity("life total exhausted");
        state.life_gained_this_turn = state
            .life_gained_this_turn
            .checked_add(amount as u128)
            .capacity("life gained this turn representation exhausted");
        state.life_gain_events_this_turn = state
            .life_gain_events_this_turn
            .checked_add(1)
            .capacity("life gain event count exhausted");
        let first = state.life_gain_events_this_turn == 1;
        let sources: Vec<_> = self
            .state
            .life_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            if source.controller != player {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::LifeGained {
                    first_only,
                    own_turn,
                } = ability.condition.event()
                    && (!first_only || first)
                    && (!own_turn || self.state.active == player)
                    && self.queue_trigger(&source, index)
                {
                    self.state.pending_triggers.last_mut().unwrap().life_event =
                        Some(LifeTriggerView { player, amount });
                }
            }
        }
    }

    pub(super) fn death_trigger_sources(&self) -> Vec<ObjectView> {
        self.state
            .death_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect()
    }

    pub(super) fn zone_triggers(
        &mut self,
        event: &ZoneChange,
        death_sources: &[ObjectView],
        sources: &[ObjectView],
        entry_suppression: u16,
    ) {
        if !event.moved {
            return;
        }
        self.delayed_target_departure(event);
        self.temporary_departure_triggers(event);
        self.forget_self_grants(event);
        if event.before.zone == Zone::Battlefield {
            // A self-departure trigger needs only its frozen pre-event source.
            // This also covers exile/bounce paths which do not collect death observers.
            let source = &event.before;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::SelfLeaves {
                    self.queue_zone_trigger(source, index, event);
                }
            }
        }
        if event.before.zone == Zone::Battlefield {
            // Departure triggers look back at the entire pre-event battlefield;
            // simultaneous departures still see each other exactly once.
            for source in death_sources {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if event.destination != Zone::Graveyard
                        && !matches!(
                            ability.condition.event(),
                            TriggerCondition::Sacrificed { .. }
                                | TriggerCondition::SacrificedByOpponent { .. }
                        )
                    {
                        continue;
                    }
                    let matched = match ability.condition.event() {
                        TriggerCondition::AttachedCreatureDies => {
                            event.before.is_creature()
                                && source.attached_to == Some(event.before.reference)
                        }
                        TriggerCondition::DamagedCreatureDies => {
                            event.before.is_creature()
                                && self
                                    .state
                                    .creature_damage_history
                                    .as_ref()
                                    .is_some_and(|rows| {
                                        rows.iter().any(|row| {
                                            row.source == source.reference
                                                && row.recipient == event.before.reference
                                        })
                                    })
                        }
                        TriggerCondition::ReturnDeath(filter) => {
                            self.return_death_matches(filter, source, event)
                        }
                        TriggerCondition::SelfCreatureDies => {
                            source.reference == event.before.reference && event.before.is_creature()
                        }
                        TriggerCondition::SelfNontokenDies => {
                            source.reference == event.before.reference && !event.before.is_token()
                        }
                        TriggerCondition::SelfDies | TriggerCondition::SelfEntersOrDies => {
                            source.reference == event.before.reference
                        }
                        TriggerCondition::CreatureDies {
                            relation,
                            other,
                            nontoken,
                            subtype,
                        } => {
                            let related = match relation {
                                CastRelation::You => source.controller == event.before.controller,
                                CastRelation::Opponent => {
                                    source.controller != event.before.controller
                                }
                                CastRelation::Any => true,
                            };
                            event.before.is_creature()
                                && related
                                && (!other || source.reference != event.before.reference)
                                && (!nontoken || event.before.token.is_none())
                                && subtype.is_none_or(|subtype| event.before.has_subtype(subtype))
                        }
                        TriggerCondition::OtherControlledPermanentWithSubtypeDies { subtypes } => {
                            source.reference != event.before.reference
                                && source.controller == event.before.controller
                                && subtypes
                                    .iter()
                                    .any(|subtype| event.before.has_subtype(subtype))
                        }
                        TriggerCondition::Sacrificed { filter: f, .. }
                        | TriggerCondition::SacrificedByOpponent { filter: f } => {
                            let (player, own_turn) = match ability.condition.event() {
                                TriggerCondition::Sacrificed { own_turn, .. } => {
                                    (source.controller, own_turn)
                                }
                                TriggerCondition::SacrificedByOpponent { .. } => {
                                    (source.controller.other(), false)
                                }
                                _ => unreachable!(),
                            };
                            event.sacrificed_by == Some(player)
                                && (!own_turn || self.state.active == source.controller)
                                && (!f.source_only || source.reference == event.before.reference)
                                && (!f.other || source.reference != event.before.reference)
                                && (f.kinds == 0 || event.before.kinds & u16::from(f.kinds) != 0)
                                && (!f.nonland || event.before.kinds & 2 == 0)
                                && event.before.keywords.contains(f.keywords)
                                && f.tokens
                                    .is_none_or(|token| event.before.is_token() == token)
                                && f.subtype
                                    .is_none_or(|subtype| event.before.has_subtype(subtype))
                        }
                        TriggerCondition::CreatureDiesWithMaximumStat { relation, maximum } => {
                            event.before.is_creature()
                                && (event.before.power <= i128::from(maximum)
                                    || event.before.toughness <= i128::from(maximum))
                                && match relation {
                                    CastRelation::You => {
                                        source.controller == event.before.controller
                                    }
                                    CastRelation::Opponent => {
                                        source.controller != event.before.controller
                                    }
                                    CastRelation::Any => true,
                                }
                        }
                        TriggerCondition::CreatureDiesFiltered { filter }
                        | TriggerCondition::CreatureDiesAttackState { filter, .. } => {
                            event.before.is_creature()
                                && match ability.condition.event() {
                                    TriggerCondition::CreatureDiesAttackState {
                                        attacking, ..
                                    } => event.before.attacking == attacking,
                                    _ => true,
                                }
                                && filter.matches_characteristics(
                                    event.before.card,
                                    event.before.is_token(),
                                    event.before.kinds,
                                    event.before.granted_subtypes,
                                    event.before.controller == source.controller,
                                    event.before.reference == source.reference,
                                    event.before.attacking,
                                )
                                && event.before.keywords.contains(filter.keyword)
                        }
                        TriggerCondition::PermanentToGraveyard {
                            kinds,
                            relation,
                            exclude_source_kinds,
                        } => {
                            let related = match relation {
                                CastRelation::You => source.controller == event.before.controller,
                                CastRelation::Opponent => {
                                    source.controller != event.before.controller
                                }
                                CastRelation::Any => true,
                            };
                            event.before.kinds & kinds != 0
                                && related
                                && (source.reference != event.before.reference
                                    || event.before.kinds & exclude_source_kinds == 0)
                        }
                        _ => false,
                    };
                    if matched && self.queue_zone_trigger(source, index, event) {
                        if matches!(ability.condition.event(), TriggerCondition::ReturnDeath(_)) {
                            self.capture_return_death(event);
                        }
                        if ability.program.iter().any(|i| {
                            matches!(
                                i,
                                Instruction::ReturnEventCardToHand
                                    | Instruction::ReturnEventCardToBattlefieldTapped
                            )
                        }) {
                            self.state.pending_triggers.last_mut().unwrap().return_link =
                                Some(Box::new(ReturnLinkView {
                                    exile_after: false,
                                    additional_returning: None,
                                    object: event.after,
                                    zone: Zone::Graveyard,
                                    controller: event.before.owner,
                                    counter: None,
                                    tapped: ability.program.iter().any(|i| {
                                        matches!(i, Instruction::ReturnEventCardToBattlefieldTapped)
                                    }),
                                }));
                        }
                        if ability.program.iter().any(|i| {
                            matches!(i, Instruction::ScheduleSacrificedArtifactCopy { .. })
                        }) {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .captured_artifact_copy = Some(Box::new(
                                event
                                    .before
                                    .copy_recipe
                                    .unwrap_or_else(|| CopyRecipe::new(event.before.card)),
                            ));
                        }
                    }
                }
            }
        }
        if event.destination != Zone::Battlefield {
            return;
        }
        // All entries in a simultaneous batch see the same post-event sources.
        let entered = self.object_view(event.after.card);
        // This prohibition concerns the event, irrespective of which permanent
        // would observe it. Death/attack/step events take their separate paths.
        if entered.kinds & entry_suppression != 0 {
            return;
        }
        self.temporary_land_entry_triggers(event, &entered);
        for source in sources {
            let same = source.reference == event.after;
            let yours = source.controller == entered.controller;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::SelfDiesWithCounters
                    | TriggerCondition::InZone { .. }
                    | TriggerCondition::ManaSpent { .. }
                    | TriggerCondition::TreasureManaSpent(_)
                    | TriggerCondition::Granted(_)
                    | TriggerCondition::OncePerTurn(_)
                    | TriggerCondition::DuringOpponentsTurn(_)
                    | TriggerCondition::When { .. }
                    | TriggerCondition::InterveningIf { .. } => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::CastFromHand(_) | TriggerCondition::CastFromExile(_) => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::SelfEntersIfCast => same && event.was_cast,
                    TriggerCondition::SelfNontokenEnters => same && !entered.is_token(),
                    TriggerCondition::SelfOffspring => same && event.offspring_paid,
                    TriggerCondition::SelfKicked => same && event.kicked,
                    TriggerCondition::SelfGiftEnters => same && event.gift.is_some(),
                    TriggerCondition::SelfBargained => same && event.bargained,
                    TriggerCondition::SelfEvidence => same && event.evidence_collected,
                    TriggerCondition::EvidenceCollected
                    | TriggerCondition::ScryOrSurveil
                    | TriggerCondition::Proliferated(_)
                    | TriggerCondition::Chapter { .. } => false,
                    TriggerCondition::SelfEnters
                    | TriggerCondition::SelfEntersOrAttacks
                    | TriggerCondition::SelfEntersOrAttacksWhileSaddled
                    | TriggerCondition::SelfEntersOrDies => same,
                    TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    | TriggerCondition::OpponentTargetsControlledCreatureOrSpell
                    | TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent
                    | TriggerCondition::TokensEnterYou
                    | TriggerCondition::CreatureAbilityActivated
                    | TriggerCondition::NonmanaAbilityActivated
                    | TriggerCondition::ExhaustAbilityActivated
                    | TriggerCondition::DelayedExhaustAbilityActivated
                    | TriggerCondition::DelayedTargetEvent
                    | TriggerCondition::LoyaltyAbilityActivated
                    | TriggerCondition::LoyaltyAbilityActivatedRemoving(_)
                    | TriggerCondition::OpponentLoyaltyAbilityActivated
                    | TriggerCondition::CardsLeaveGraveyard
                    | TriggerCondition::CardsLeaveGraveyardKinds(_)
                    | TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                    | TriggerCondition::CreatureCardLeavesYourGraveyard
                    | TriggerCondition::PermanentCardsEnterYourGraveyard
                    | TriggerCondition::CreatureCardsEnterYourGraveyard
                    | TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    | TriggerCondition::SelfExiledWhileCrafting
                    | TriggerCondition::CreatureExploresYou
                    | TriggerCondition::SelfTurnedFaceUp
                    | TriggerCondition::PermanentYouControlTurnedFaceUp
                    | TriggerCondition::YouManifestDread
                    | TriggerCondition::CreatureExploresCardYou { .. }
                    | TriggerCondition::Foraged
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Reflexive
                    | TriggerCondition::DelayedSpellCastSourcePower
                    | TriggerCondition::Ward
                    | TriggerCondition::SelfTargeted { .. }
                    | TriggerCondition::CardDrawn { .. }
                    | TriggerCondition::LibrarySearched { .. }
                    | TriggerCondition::BecomesClassLevel(_)
                    | TriggerCondition::SelfDiscarded
                    | TriggerCondition::SelfCycled
                    | TriggerCondition::Discarded { .. }
                    | TriggerCondition::ControlledCreatureCombatDamageOwnTurn
                    | TriggerCondition::SourceDealtCombatDamage
                    | TriggerCondition::AttachedDealtCombatDamage
                    | TriggerCondition::DamagedCreatureDies
                    | TriggerCondition::StateCounterAtLeast { .. }
                    | TriggerCondition::CounterThresholdCrossed { .. }
                    | TriggerCondition::PlayerDamaged { .. }
                    | TriggerCondition::CreatureDamaged { .. }
                    | TriggerCondition::SelfDamaged
                    | TriggerCondition::SelfDies
                    | TriggerCondition::SelfNontokenDies
                    | TriggerCondition::SelfCreatureDies
                    | TriggerCondition::SelfLeaves
                    | TriggerCondition::ReturnDeath(_)
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::OtherControlledPermanentWithSubtypeDies { .. }
                    | TriggerCondition::AttachedCreatureDies
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesWithMaximumStat { .. }
                    | TriggerCondition::CreatureDiesBatch { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::CreatureDiesAttackState { .. }
                    | TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::SacrificedByOpponent { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::LifeLost { .. }
                    | TriggerCondition::SelfCountersAdded { .. }
                    | TriggerCondition::CreatureAttacksYou { .. }
                    | TriggerCondition::CreaturesAttackYou
                    | TriggerCondition::OtherCreatureCountersAdded { .. }
                    | TriggerCondition::OtherCreatureCounterPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYou { .. }
                    | TriggerCondition::CounterBatchPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                    | TriggerCondition::AttackDeclared { .. }
                    | TriggerCondition::FirstAttackWithTotalPowerAtLeast(_)
                    | TriggerCondition::AttackPlayer(_)
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SelfAttacksIfEquipped
                    | TriggerCondition::SelfFirstAttack
                    | TriggerCondition::SelfAttacksAlone
                    | TriggerCondition::ControlledCreatureAttacksAlone
                    | TriggerCondition::ControlledCreatureAttacks
                    | TriggerCondition::ControlledCreatureAttacksPlayerAlone
                    | TriggerCondition::OpponentCreatureBlocksSmall
                    | TriggerCondition::ControlledEquippedCreatureAttacks
                    | TriggerCondition::SelfAttacksWhileSaddled
                    | TriggerCondition::SelfSaddledFirst
                    | TriggerCondition::SelfCrewsOrSaddles
                    | TriggerCondition::SelfPaysTeamwork
                    | TriggerCondition::EquippedAttacks
                    | TriggerCondition::EquippedAttacksAlone
                    | TriggerCondition::GrantedAttacks
                    | TriggerCondition::AttachedAttacksOrBlocks
                    | TriggerCondition::AttachedCreatureBecomesTapped
                    | TriggerCondition::AttachedBecomesUntapped
                    | TriggerCondition::SelfBecomesTapped
                    | TriggerCondition::ControlledCreaturesBecomeTapped
                    | TriggerCondition::ControlledCreatureFirstTapOwnTurn
                    | TriggerCondition::SelfCast
                    | TriggerCondition::SpellCast { .. }
                    | TriggerCondition::ManaSpentForSpell
                    | TriggerCondition::AtStep { .. } => false,
                    TriggerCondition::PermanentEntersYou {
                        kinds,
                        nontoken,
                        subtype,
                        include_source,
                    } => {
                        yours
                            && if same {
                                include_source
                            } else {
                                (kinds == 0 || entered.kinds & kinds != 0)
                                    && (!nontoken || entered.token.is_none())
                                    && subtype.is_none_or(|t| entered.has_subtype(t))
                            }
                    }
                    TriggerCondition::OtherFilteredEntersYou(predicate)
                    | TriggerCondition::FilteredEntersYou(predicate) => {
                        yours
                            && (!same
                                || matches!(
                                    ability.condition.event(),
                                    TriggerCondition::FilteredEntersYou(_)
                                ))
                            && match predicate {
                                EntryPredicate::Token => entered.is_token(),
                                EntryPredicate::LegendarySubtype(subtype) => {
                                    entered.is_legendary() && entered.has_subtype(subtype)
                                }
                                EntryPredicate::ArtifactManaValueAtLeast(minimum) => {
                                    entered.kinds & 16 != 0
                                        && self.object_mana_value(entered.id) >= u64::from(minimum)
                                }
                                EntryPredicate::CreatureWithoutSubtype(subtype) => {
                                    entered.is_creature() && !entered.has_subtype(subtype)
                                }
                                EntryPredicate::CreaturePowerAtLeast(minimum) => {
                                    entered.is_creature() && entered.power >= i128::from(minimum)
                                }
                                EntryPredicate::CreaturePowerAtMost(maximum) => {
                                    entered.is_creature() && entered.power <= i128::from(maximum)
                                }
                                EntryPredicate::SubtypeAny(subtypes) => {
                                    subtypes.iter().any(|subtype| entered.has_subtype(subtype))
                                }
                                EntryPredicate::KindOrSubtype { kinds, subtype } => {
                                    entered.kinds & kinds != 0 || entered.has_subtype(subtype)
                                }
                            }
                    }
                    TriggerCondition::OtherCreatureEntersYouIfGreaterStats => {
                        yours
                            && !same
                            && CreatureComparisonFacts::from_view(&entered)
                                .greater_than(CreatureComparisonFacts::from_view(source))
                    }
                    TriggerCondition::OtherCreatureEntersYou => {
                        yours && !same && entered.is_creature()
                    }
                    TriggerCondition::CreatureEnters { relation, other } => {
                        entered.is_creature()
                            && (!other || !same)
                            && match relation {
                                CastRelation::You => yours,
                                CastRelation::Opponent => !yours,
                                CastRelation::Any => true,
                            }
                    }
                    TriggerCondition::OtherSubtypeEntersYou(subtype) => {
                        yours && !same && entered.has_subtype(subtype)
                    }
                    TriggerCondition::SubtypeEntersYou(subtype) => {
                        yours && entered.has_subtype(subtype)
                    }
                    TriggerCondition::SubtypeEntersYouWithOthers { subtype, minimum } => {
                        yours
                            && entered.has_subtype(subtype)
                            && self.other_controlled_subtypes_at_least(
                                source.controller,
                                event.after,
                                subtype,
                                minimum,
                            )
                    }
                    TriggerCondition::ArtifactEntersYou => yours && entered.kinds & 16 != 0,
                    TriggerCondition::EnchantmentEntersYou => yours && entered.kinds & 32 != 0,
                    TriggerCondition::CreatureEntersYou => yours && entered.is_creature(),
                    TriggerCondition::LandEntersYou => yours && entered.kinds & 2 != 0,
                    TriggerCondition::OtherArtifactEntersYou => {
                        yours && !same && entered.kinds & 16 != 0
                    }
                };
                if matched && self.queue_zone_trigger(source, index, event) {
                    if ability.program.iter().any(|instruction| {
                        matches!(
                            instruction,
                            Instruction::DamagePlayers(PlayerSet::EventPlayer, _)
                        )
                    }) {
                        self.state.pending_triggers.last_mut().unwrap().event_player =
                            Some(entered.controller);
                    }
                    if ability.condition.event()
                        == TriggerCondition::OtherCreatureEntersYouIfGreaterStats
                    {
                        self.bind_creature_comparison(source, &entered);
                    }
                    if same {
                        self.state.pending_triggers.last_mut().unwrap().cast_event =
                            event.cast_event.clone();
                    }
                    self.bind_stat_subject(event.after, source.card.triggers()[index].program);
                    self.bind_entering_conniver(&entered, source.card.triggers()[index].program);
                    self.bind_entering_copy(&entered, source.card.triggers()[index].program);
                    self.bind_entering_subtype_lki(&entered, source.card.triggers()[index].program);
                    if same && source.card.x_symbols() > 0 {
                        self.state.pending_triggers.last_mut().unwrap().x = Some(event.x);
                    }
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entering_object = Some(event.after);
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .evidence_collected = event.evidence_collected;
                }
            }
        }
    }

    fn queue_zone_trigger(
        &mut self,
        source: &ObjectView,
        index: usize,
        event: &ZoneChange,
    ) -> bool {
        self.queue_zone_trigger_from(source, index, event, None)
    }

    pub(super) fn craft_exile_triggers(&mut self, event: &ZoneChange, actor: Player) {
        if !event.moved
            || event.before.zone != Zone::Battlefield
            || event.destination != Zone::Exile
            || event.before.controller != actor
        {
            return;
        }
        for (index, ability) in event.before.card.triggers().iter().enumerate() {
            if ability.condition.event() == TriggerCondition::SelfExiledWhileCrafting {
                self.queue_zone_trigger(&event.before, index, event);
            }
        }
    }

    pub(super) fn queue_zone_trigger_from(
        &mut self,
        source: &ObjectView,
        index: usize,
        event: &ZoneChange,
        grant: Option<Box<AttachmentTriggerView>>,
    ) -> bool {
        if !self.queue_trigger_from(source, index, grant) {
            return false;
        }
        if event.moved
            && event.destination == Zone::Battlefield
            && source.reference == event.after
            && let Some(context) = &event.gift
            && context.source == event.before.reference
        {
            let mut captured = **context;
            captured.entry = Some(event.after);
            self.state.pending_triggers.last_mut().unwrap().gift = Some(Box::new(captured));
        }
        // Only the movement which actually triggered this ability may bind its
        // new source. Unrelated simultaneous observer deaths do not qualify.
        if event.moved
            && event.before.reference == source.reference
            && matches!(
                event.destination,
                Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
            )
        {
            self.state.pending_triggers.last_mut().unwrap().source_move =
                Some(Box::new(TriggerSourceMoveView {
                    before: event.before.reference,
                    after: event.after,
                    origin: event.before.zone,
                    destination: event.destination,
                }));
        }
        true
    }

    pub(super) fn queue_trigger(&mut self, source: &ObjectView, index: usize) -> bool {
        self.queue_trigger_from(source, index, None)
    }

    pub(super) fn queue_trigger_from(
        &mut self,
        source: &ObjectView,
        index: usize,
        event: Option<Box<AttachmentTriggerView>>,
    ) -> bool {
        self.remember_hideaway_controller(source.reference, source.controller);
        if let Some(grant) = event.as_ref().filter(|grant| grant.granted) {
            if !source.grant_available(
                grant.grantor,
                grant.granting_card,
                u16::try_from(index).expect("ability index"),
                None,
            ) {
                return false;
            }
        } else if !source.printed_abilities_available() {
            return false;
        }
        let definition = event
            .as_ref()
            .filter(|event| event.granted)
            .map_or(source.card, |event| event.granting_card);
        let ability_origin = event.as_ref().filter(|event| event.granted).map_or_else(
            || {
                source
                    .temporary_copy
                    .as_ref()
                    .map(|view| view.active_origin)
            },
            |event| event.origin,
        );
        let condition = definition.triggers()[index].condition;
        if condition.event() == TriggerCondition::SelfAttacksIfEquipped
            && self.source_equipment_count(source.reference).unwrap_or(0) == 0
        {
            return false;
        }
        if condition == TriggerCondition::SelfDiesWithCounters && !source.counters.has_any() {
            return false;
        }
        // Graveyard observers must not enable unqualified battlefield abilities.
        // Self-discard/cycling dispatch deliberately uses the destination object,
        // so those events remain valid there. An explicit origin always wins.
        let destination_event = matches!(
            condition.event(),
            TriggerCondition::SelfDiscarded | TriggerCondition::SelfCycled
        );
        let origin_matches = condition.origin().map_or(
            source.zone != Zone::Graveyard || destination_event,
            |origin| {
                source.zone
                    == match origin {
                        TargetZone::Battlefield => Zone::Battlefield,
                        TargetZone::Graveyard => Zone::Graveyard,
                    }
            },
        );
        if !origin_matches {
            return false;
        }
        if condition.is_granted() != event.as_ref().is_some_and(|e| e.granted) {
            return false;
        }

        if !self.trigger_time_guards_hold(
            condition,
            source.controller,
            source.reference,
            source.tapped,
        ) || !self.trigger_guards_hold(
            condition,
            source.controller,
            source.reference,
            source.tapped,
        ) {
            return false;
        }
        if matches!(
            definition.triggers()[index].program.first(),
            Some(Instruction::MayOncePerTurn { .. })
        ) && self.state.triggered_this_turn.contains(&TriggerUse {
            source: source.reference,
            ability: u16::try_from(index).expect("ability index"),
            origin: ability_origin,
        }) {
            return false;
        }
        if condition.once_per_turn() {
            let used = TriggerUse {
                source: source.reference,
                ability: u16::try_from(index).expect("ability index"),
                origin: ability_origin,
            };
            if self.state.triggered_this_turn.contains(&used) {
                return false;
            }
            // The ability has triggered even if its later announcement has no
            // legal targets, is countered, or its effect is declined.
            self.state.triggered_this_turn.push(used);
        }
        let id = self.next_stack_id();
        self.state.pending_triggers.push(SpellView {
            source: Some(StackSourceView {
                id: source.id,
                object: source.reference,
                card: source.card,
                facts: DamageFacts::from_view(source),
            }),
            source_less_rule: None,
            // Explicit limits and state-trigger latching also cap extra instances.
            additional_triggers: if condition.once_per_turn()
                || matches!(
                    condition.event(),
                    TriggerCondition::StateCounterAtLeast { .. }
                ) {
                0
            } else {
                source.additional_triggers
            },
            gift: None,
            source_move: None,
            borrowed_slot: None,
            self_granted_activation: None,
            target_controller_lki: None,
            crime_qualifying_target: false,
            target_kinds_lki: None,
            target_type_slots: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            target_allocations: None,
            captured_artifact_copy: None,
            source_copy_recipe: source
                .copy_recipe
                .filter(|r| {
                    r.exceptions != CopyExceptions::NONE
                        || r.overrides.is_some()
                        || r.faces.is_some()
                })
                .map(Box::new),
            source_ability_origin: ability_origin,
            mana_copy_sources: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            loyalty_x: None,
            reflexive_target_limit: None,
            attachment_trigger: event,
            cast: None,
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_transform: None,

            split_second: false,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            mana_tap_bonus: None,
            related_stack: None,
            entering_object: None,
            evidence_collected: false,
            damage_event: None,
            damage_received: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_payment: None,
            loyalty_counters_paid: 0,
            exiled_symbol_cards: None,
            craft_materials: None,
            paid_additional_cost: None,
            exiled_trigger_cards: None,
            source_counter_lki: {
                let mut facts = card_quantities::counter_quantity_facts(
                    definition.triggers()[index].program,
                    |kind| source.counters.get(kind),
                );
                // An intervening-if may read a counter even when its program
                // does not. Track that kind on the ordinary trigger packet so
                // a departure freezes the exact source's last known value.
                if let TriggerCondition::InterveningIf {
                    guard:
                        StepGuard::Expression {
                            value: StatValue::SourceCounters(kind),
                            ..
                        },
                    ..
                } = condition
                {
                    let rows = facts.get_or_insert_with(Default::default);
                    if !rows.iter().any(|row| row.kind == kind) {
                        rows.push(card_quantities::SourceCounterQuantityView {
                            kind,
                            count: source.counters.get(kind),
                        });
                    }
                }
                facts
            },
            event_player: None,
            event_object: None,
            manifest_event: None,
            source_quantity_lki: self
                .source_quantity_facts(definition.triggers()[index].program, source),
            sacrificed_toughness: None,
            sacrificed_mana_value: None,
            sacrificed_power: None,
            source_attachment: source.attached_to,

            controller: source.controller,
            targets: Vec::new(),
            mode: None,
        });
        true
    }

    /// Put a direct activated ability on the stack before paying its costs.
    pub(super) fn begin_direct_activation_on_stack(&mut self, item: &mut SpellView) {
        self.capture_transform_stack_serial(item);
        self.state.stack.push(item.clone());
    }

    /// Complete an already-announced direct activation without inserting it twice.
    pub(super) fn finish_direct_activation_on_stack(&mut self, item: SpellView) {
        if let StackKind::ActivatedAbility { ability } = item.kind
            && !item.is_copy
            && item
                .source
                .is_some_and(|source| source.card.activation(ability).is_equip())
        {
            self.state.players[item.controller.index()].equip_activated_this_turn = true;
        }
        let source_still_exact = item.source.is_some_and(|source| {
            self.state
                .objects
                .get(source.id as usize)
                .is_some_and(|object| object.generation == source.object.generation)
        });
        let slot = self
            .state
            .stack
            .iter_mut()
            .find(|entry| entry.id == item.id)
            .expect("announced activation remains on the stack while costs are paid");
        assert_eq!(
            slot.source.map(|source| source.object),
            item.source.map(|source| source.object)
        );
        // Departure and event walkers update the on-stack packet while the
        // cost executor temporarily owns a local packet. Keep those facts and
        // copy back only the fields that payment itself owns.
        slot.loyalty_counters_paid = item.loyalty_counters_paid;
        slot.source_counter_payment = item.source_counter_payment;
        slot.mana_tap_bonus = item.mana_tap_bonus;
        slot.source_move = item.source_move;
        slot.paid_additional_cost = item.paid_additional_cost;
        slot.crime_qualifying_target = item.crime_qualifying_target;
        if source_still_exact {
            slot.source_counter_lki = item.source_counter_lki;
        }
        let completed = slot.clone();
        self.record_committed_crime(&completed);
        self.committed_activation_observers(&completed);
    }

    /// A spell becomes cast only after every declaration and cost is complete.
    /// Completed insertion paths share this hook; abilities never count as casts.
    pub(super) fn push_stack_item(&mut self, mut item: SpellView) {
        self.capture_transform_stack_serial(&mut item);
        if !item.is_copy
            && let StackKind::ActivatedAbility { ability } = item.kind
            && item
                .source
                .is_some_and(|source| source.card.activation(ability).is_equip())
        {
            self.state.players[item.controller.index()].equip_activated_this_turn = true;
        }
        let cast = (item.kind == StackKind::Spell && item.cast.is_some() && !item.is_copy)
            .then(|| {
                item.source
                    .map(|source| (item.controller, source.card, source.object))
            })
            .flatten();
        if let Some((_, _, object)) = cast {
            self.complete_prepared_cast(object);
        }
        self.register_stack_graveyard_exile(&item);
        self.record_committed_crime(&item);
        self.committed_activation_observers(&item);
        self.state.stack.push(item);
        let Some((caster, card, _)) = cast else {
            return;
        };
        self.record_cast(caster, card, Some(self.state.stack.len() - 1));
    }

    /// CR115.9b: inspect the current expected zone/incarnation; never use target LKI.
    #[allow(
        clippy::too_many_arguments,
        reason = "Cast filters need distinct spell owner, caster, value, source and exact target context"
    )]
    pub(super) fn cast_filter_matches_value(
        &self,
        spell_owner: Option<Player>,
        filter: CastFilter,
        card: Card,
        mana_value: u64,
        caster: Player,
        source: &ObjectView,
        targets: &[BoundTarget],
    ) -> bool {
        match filter {
            CastFilter::NotOwnedByCaster => spell_owner.is_some_and(|owner| owner != caster),
            CastFilter::CasterOffTurn { filter } => {
                self.state.active != caster
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                    )
            }
            CastFilter::NotOwnedByYou => {
                spell_owner.is_some_and(|owner| owner != source.controller)
            }
            CastFilter::SourceChosenManaValueParity => {
                matches!(source.entry_choice, Some(EntryChoiceValue::ManaValueParity(parity))
                    if parity.matches(mana_value))
            }
            CastFilter::SourceChosenColor => {
                matches!(source.entry_choice, Some(EntryChoiceValue::Color(color)) if card.colors() & (1 << color.index()) != 0)
            }
            CastFilter::NonCreatureOrdinal(wanted) => {
                let player = &self.state.players[caster.index()];
                CastFilter::Noncreature.matches(card)
                    && player
                        .spells_cast_this_turn
                        .checked_sub(player.creature_spells_cast_this_turn)
                        .is_some_and(|count| count == wanted)
            }
            CastFilter::InstantSorceryOrdinal(wanted) => {
                CastFilter::InstantSorcery.matches(card)
                    && self.state.players[caster.index()].instant_sorcery_cast_this_turn == wanted
            }
            CastFilter::Colors { filter, mask } => {
                card.colors() & mask != 0
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                    )
            }
            CastFilter::ManaSymbols { filter, color } => {
                card.mana_symbols_of_color(color) > 0
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                    )
            }
            CastFilter::AnyOf(filters) => filters.iter().any(|f| {
                self.cast_filter_matches_value(
                    spell_owner,
                    *f,
                    card,
                    mana_value,
                    caster,
                    source,
                    targets,
                )
            }),
            CastFilter::ManaValueMaximum { filter, maximum } => {
                mana_value <= maximum
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                    )
            }
            CastFilter::ManaValueMinimum { filter, minimum } => {
                mana_value >= minimum
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                    )
            }
            CastFilter::Targeted { filter, target } => {
                self.cast_filter_matches_value(
                    spell_owner,
                    *filter,
                    card,
                    mana_value,
                    caster,
                    source,
                    targets,
                ) && targets.iter().any(|bound| {
                    if let Target::Player(player) = bound.target {
                        return target == CastTarget::OpponentOrCreature
                            && player != source.controller;
                    }
                    let (Target::Card(id), Some(reference)) = (bound.target, bound.object) else {
                        return false;
                    };
                    if id != reference.card
                        || !self.on_battlefield(id)
                        || self.object_ref(id) != reference
                    {
                        return false;
                    }
                    let object = &self.state.objects[id as usize];
                    match target {
                        CastTarget::Source => reference == source.reference,
                        CastTarget::PermanentKinds(kinds) => self.current_kinds(id) & kinds != 0,
                        CastTarget::OpponentOrCreature => {
                            self.is_creature(id) && object.controller != source.controller
                        }
                        CastTarget::Creature(relation) => {
                            self.is_creature(id)
                                && match relation {
                                    CastRelation::You => object.controller == source.controller,
                                    CastRelation::Opponent => {
                                        object.controller != source.controller
                                    }
                                    CastRelation::Any => true,
                                }
                        }
                    }
                })
            }
            _ => filter.matches(card),
        }
    }

    pub(super) fn record_cast(&mut self, caster: Player, card: Card, stack: Option<usize>) {
        // Every committed path classifies the selected spell face, including
        // manual direct-payment insertion. The physical front may differ.
        let card = stack
            .and_then(|i| self.state.stack[i].source.map(|source| source.card))
            .unwrap_or(card);
        if let Some(index) = stack {
            let item = self.state.stack[index].clone();
            self.expire_land_mana_on_exile_cast(&item);
        }
        self.consume_resolution_cast_quota(caster, stack);
        self.consume_next_spell_uncounterable(caster, stack);
        self.consume_turn_cast_grants(caster, card);
        let first_trigger = self.state.pending_triggers.len();
        let spell_owner = stack.and_then(|i| {
            self.state.stack[i]
                .source
                .and_then(|source| self.state.objects.get(source.id as usize))
                .map(|object| object.owner)
        });
        let count = &mut self.state.players[caster.index()].spells_cast_this_turn;
        *count = count.checked_add(1).capacity("spell count exhausted");
        let ordinal = *count;
        if CastFilter::InstantSorcery.matches(card) {
            let count = &mut self.state.players[caster.index()].instant_sorcery_cast_this_turn;
            *count = count
                .checked_add(1)
                .capacity("qualifying spell count exhausted");
        }
        if CastFilter::Creature.matches(card) {
            let count = &mut self.state.players[caster.index()].creature_spells_cast_this_turn;
            *count = count
                .checked_add(1)
                .capacity("creature spell count exhausted");
        }
        let cast_event = stack.and_then(|i| {
            let item = &self.state.stack[i];
            item.cast
                .zip(item.source)
                .map(|(cast, source)| CastPaymentEvent {
                    mana_value: item.mana_value(),
                    stack_id: item.id,
                    object: source.object,
                    card: source.card,
                    controller: item.controller,
                    payment: cast.payment,
                    treasure_mana_spent: cast.treasure_mana_spent,
                })
        });
        let sources = self.indexed_trigger_sources(&self.state.cast_sources);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if let TriggerCondition::SpellCast {
                    relation,
                    filter,
                    ordinal: wanted,
                } = ability.condition.event()
                {
                    let related = match relation {
                        CastRelation::You => source.controller == caster,
                        CastRelation::Opponent => source.controller != caster,
                        CastRelation::Any => true,
                    };
                    if related
                        && (!ability.condition.requires_exile_cast()
                            || stack.is_some_and(|i| {
                                self.state.stack[i]
                                    .cast
                                    .is_some_and(|c| c.origin == Zone::Exile)
                            }))
                        && (!ability.condition.requires_hand_cast()
                            || stack.is_some_and(|i| {
                                self.state.stack[i]
                                    .cast
                                    .is_some_and(|c| c.origin == Zone::Hand)
                            }))
                        && self.cast_payment_condition(
                            ability.condition,
                            source.reference,
                            cast_event.as_ref(),
                        )
                        && (!ability.condition.requires_opponents_turn()
                            || self.state.active != source.controller)
                        && (if filter == CastFilter::PreparedSpell {
                            stack.is_some_and(|i| {
                                let spell = &self.state.stack[i];
                                spell
                                    .source
                                    .is_some_and(|source| source.card.is_prepared_face())
                                    && spell.source.is_some_and(|source| {
                                        self.state.objects[source.id as usize].is_spell_copy()
                                    })
                            })
                        } else if filter == CastFilter::Colorless {
                            stack
                                .and_then(|i| self.state.stack[i].source)
                                .map_or(card.colors() == 0, |source| source.facts.colors == 0)
                        } else {
                            self.cast_filter_matches_value(
                                spell_owner,
                                filter,
                                card,
                                stack
                                    .map(|i| self.state.stack[i].mana_value())
                                    .unwrap_or_else(|| card.fixed_mana_value()),
                                caster,
                                &source,
                                stack
                                    .map(|i| self.state.stack[i].targets.as_slice())
                                    .unwrap_or(&[]),
                            )
                        })
                        && wanted.is_none_or(|wanted| wanted == ordinal)
                        && self.queue_trigger_from(&source, index, grant.map(Box::new))
                    {
                        // A cast trigger owns the triggering spell's chosen X even after that
                        // spell is countered or leaves the stack. ChosenX reads this packet.
                        let chosen_x = stack.and_then(|i| self.state.stack[i].x);
                        self.state.pending_triggers.last_mut().unwrap().x = chosen_x;
                        if ability.condition.mana_spent_rule().is_some()
                            || ability.condition.requires_treasure_mana()
                            || cast_symbol_quantities::selected(ability.program)
                            || ability.program.iter().any(|instruction| {
                                matches!(
                                    instruction,
                                    Instruction::ChangeCastSpellTargets
                                        | Instruction::GainLifeScaled(_, AmountValue::SpellColors)
                                        | Instruction::DamageScaled(
                                            AmountValue::CastSpellManaValue
                                        )
                                        | Instruction::CreateTokensWithTemporaryKeywords { .. }
                                        | Instruction::DamagePlayers(PlayerSet::CastingPlayer, _)
                                        | Instruction::CreateTokensFor {
                                            players: PlayerSet::CastingPlayer,
                                            ..
                                        }
                                        | Instruction::PutCountersScaled {
                                            amount: AmountValue::SpellColors
                                                | AmountValue::CastSpellManaValue,
                                            ..
                                        }
                                )
                            })
                        {
                            self.state.pending_triggers.last_mut().unwrap().cast_event =
                                cast_event.map(Box::new);
                        }
                    }
                }
            }
        }
        // Source-cast abilities live on the actual stack spell, outside the
        // battlefield observer index. Capture copies below before any priority.
        if let Some(index) = stack {
            let item = &self.state.stack[index];
            if !item.is_copy {
                let source = self.object_view(item.source.expect("cast spell has source").id);
                for (ability, trigger) in source.card.triggers().iter().enumerate() {
                    if trigger.condition.event() == TriggerCondition::SelfCast
                        && self.queue_trigger(&source, ability)
                    {
                        self.state.pending_triggers.last_mut().unwrap().cast_event =
                            cast_event.map(Box::new);
                    }
                }
            }
        }
        if let Some(index) = stack {
            self.queue_hand_cascade(index);
            self.queue_instant_sorcery_storm(index);
        }
        self.temporary_cast_triggers(caster, card, stack);
        self.capture_cast_creature_subjects(stack, first_trigger);
        self.capture_copy_cast(stack, first_trigger);
        if let Some(index) = stack {
            self.capture_mana_copies(index);
        }
    }

    /// Apply one simultaneous tap event, then collect source-owned triggers.
    /// Entry status and virtual mana funding never use this transition.
    pub(super) fn tap_objects(&mut self, ids: &[CardId]) {
        // Mana taps and single-target effects are the common path. Record exact
        // turn history without evaluating views when no observer needs them.
        if let &[id] = ids {
            let object = &mut self.state.objects[id as usize];
            if object.zone == Zone::Battlefield && !object.tapped {
                object.tapped = true;
                self.record_tap(self.object_ref(id));
                self.tapped_triggers(id);
                self.tapped_batch_triggers(&[id]);
            }
            return;
        }
        let mut changed = Vec::new();
        for &id in ids {
            let object = &mut self.state.objects[id as usize];
            if object.zone == Zone::Battlefield && !object.tapped {
                object.tapped = true;
                changed.push(id);
            }
        }
        // Every observer sees the complete simultaneous tapped result. No queued
        // effect resolves until the surrounding action or instruction completes.
        for &id in &changed {
            self.record_tap(self.object_ref(id));
        }
        for &id in &changed {
            self.tapped_triggers(id);
        }
        self.tapped_batch_triggers(&changed);
    }

    fn tapped_batch_triggers(&mut self, changed: &[CardId]) {
        self.attached_tap_triggers(changed);
        if changed.is_empty() || self.state.tap_sources.is_empty() {
            return;
        }
        let sources: Vec<_> = self
            .state
            .tap_sources
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            if !changed.iter().any(|&id| {
                let object = &self.state.objects[id as usize];
                object.controller == source.controller && self.object_kinds(id, object) & 1 != 0
            }) {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                match ability.condition.event() {
                    TriggerCondition::ControlledCreaturesBecomeTapped => {
                        self.queue_trigger(&source, index);
                    }
                    TriggerCondition::ControlledCreatureFirstTapOwnTurn
                        if self.state.active == source.controller =>
                    {
                        for &id in changed {
                            let object = &self.state.objects[id as usize];
                            let reference = self.object_ref(id);
                            if object.controller == source.controller
                                && self.object_kinds(id, object) & 1 != 0
                                && self.first_tap(reference)
                                && self.queue_trigger(&source, index)
                            {
                                self.state
                                    .pending_triggers
                                    .last_mut()
                                    .unwrap()
                                    .entering_object = Some(reference);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn tapped_triggers(&mut self, id: CardId) {
        let card = self.state.objects[id as usize].card;
        if !card.triggers().iter().any(|ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::SelfBecomesTapped
            )
        }) {
            return;
        }
        let source = self.object_view(id);
        for (index, ability) in card.triggers().iter().enumerate() {
            if matches!(
                ability.condition.event(),
                TriggerCondition::SelfBecomesTapped
            ) {
                self.queue_trigger(&source, index);
            }
        }
    }

    /// One event per completed declaration, irrespective of whether its observer attacked.
    pub(super) fn declaration_triggers(
        &mut self,
        actor: Player,
        alone: Option<ObjectRef>,
        declared: &[AttackDeclaration],
        power_event: turn_facts::AttackPowerEvent,
    ) {
        self.attachment_attack_triggers(actor, alone);
        self.temporary_attack_triggers(actor);
        self.defender_attack_group_triggers(declared);
        let delayed = self
            .state
            .delayed_opponent_attack_until_next_turn
            .as_deref()
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        for mut packet in delayed {
            let StackKind::TriggeredAbility { ability } = packet.kind else {
                continue;
            };
            let Some(trigger) = packet
                .trigger_definition()
                .triggers()
                .get(usize::from(ability))
            else {
                continue;
            };
            let each_defender = matches!(
                trigger.program.first(),
                Some(Instruction::WeakenDeclaredAttacker(_))
            );
            if each_defender {
                if actor == packet.controller {
                    continue;
                }
                for attack in declared {
                    let defended = match attack.defender {
                        Some(CombatDefender::Player(player)) => player == packet.controller,
                        Some(CombatDefender::Planeswalker(object)) => {
                            self.on_battlefield(object.card)
                                && self.object_ref(object.card) == object
                                && self.state.objects[object.card as usize].controller
                                    == packet.controller
                        }
                        _ => false,
                    };
                    if defended {
                        let mut trigger = packet.clone();
                        trigger.id = self.next_stack_id();
                        trigger.reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                            object: self.object_ref(attack.card),
                            additional_objects: Vec::new(),
                        }));
                        self.state.pending_triggers.push(trigger);
                    }
                }
            } else if packet.controller == actor {
                let attackers = declared
                    .iter()
                    .filter(|attack| attack.defender == Some(CombatDefender::Player(actor.other())))
                    .map(|attack| self.object_ref(attack.card))
                    .collect::<Vec<_>>();
                if let Some((&first, rest)) = attackers.split_first() {
                    packet.id = self.next_stack_id();
                    packet.reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                        object: first,
                        additional_objects: rest.to_vec(),
                    }));
                    self.state.pending_triggers.push(packet);
                }
            }
        }
        if !declared.is_empty() {
            let packets: Vec<_> = self
                .state
                .delayed_attack_this_turn
                .as_deref()
                .into_iter()
                .flatten()
                .filter(|item| item.controller == actor)
                .cloned()
                .collect();
            for mut packet in packets {
                packet.id = self.next_stack_id();
                self.state.pending_triggers.push(packet);
            }
        }
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id) && self.state.objects[id as usize].controller == actor
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::FirstAttackWithTotalPowerAtLeast(minimum) =
                    ability.condition.event()
                    && power_event
                        .previous_maximum
                        .is_none_or(|prior| prior < i128::from(minimum))
                    && power_event.total >= i128::from(minimum)
                {
                    self.queue_trigger(&source, index);
                }
                if ability.condition.event() == TriggerCondition::ControlledCreatureAttacksAlone
                    && let Some(attacker) = alone
                    && self.queue_trigger(&source, index)
                {
                    // This existing field holds a generic exact event object.
                    // Capture the declared attacker, not a target or later combat count.
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entering_object = Some(attacker);
                }
                if ability.condition.event() == TriggerCondition::ControlledCreatureAttacks {
                    for attacker in declared {
                        let object = self.object_ref(attacker.card);
                        let facts =
                            EventObjectFacts::DamageActor(self.damage_source(attacker.card));
                        if self.queue_trigger(&source, index) {
                            self.state.pending_triggers.last_mut().unwrap().event_object =
                                Some(Box::new(EventObjectView {
                                    object,
                                    facts,
                                    conniver_creature: None,
                                    subtype_lki: None,
                                }));
                        }
                    }
                }
                if ability.condition.event()
                    == TriggerCondition::ControlledCreatureAttacksPlayerAlone
                    && let Some(attacker) = alone
                    && matches!(declared[0].defender, Some(CombatDefender::Player(_)))
                    && self.queue_trigger(&source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entering_object = Some(attacker);
                }
                if ability.condition.event() == TriggerCondition::ControlledEquippedCreatureAttacks
                {
                    for attacker in declared {
                        let event = self.object_view(attacker.card);
                        if self.source_equipment_count(event.reference).unwrap_or(0) != 0
                            && self.queue_trigger(&source, index)
                        {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .entering_object = Some(event.reference);
                            self.bind_entering_conniver(&event, ability.program);
                        }
                    }
                }
                if let TriggerCondition::CreatureAttacksYou { colors } = ability.condition.event() {
                    let count = self
                        .state
                        .attackers
                        .iter()
                        .filter(|attacker| {
                            self.state.objects[attacker.card as usize].controller == actor
                                && ability_world::CharacteristicWorld::colors(self, attacker.card)
                                    & colors
                                    != 0
                        })
                        .count();
                    for _ in 0..count {
                        self.queue_trigger(&source, index);
                    }
                }
                if let TriggerCondition::AttackPlayer(filter) = ability.condition.event() {
                    // Qualification belongs to this declaration, not all live attackers.
                    // A creature put onto the battlefield attacking did not attack.
                    for player in [Player::P0, Player::P1] {
                        if declared.iter().any(|attack| {
                            attack.defender == Some(CombatDefender::Player(player))
                                && self.creature_matches(
                                    filter,
                                    actor,
                                    source.reference,
                                    attack.card,
                                )
                        }) {
                            self.queue_trigger(&source, index);
                        }
                    }
                }
                if let TriggerCondition::AttackDeclared {
                    minimum,
                    power_min,
                    guard,
                } = ability.condition.event()
                {
                    let count = self
                        .state
                        .attackers
                        .iter()
                        .filter(|a| {
                            let o = &self.state.objects[a.card as usize];
                            o.controller == actor
                                && o.zone == Zone::Battlefield
                                && self.is_creature(a.card)
                                && power_min
                                    .is_none_or(|p| self.object_view(a.card).power >= i128::from(p))
                        })
                        .count();
                    // This is an event qualification, never an intervening-if recheck.
                    if count >= usize::from(minimum)
                        && self.step_guard(guard, actor, source.reference, source.tapped)
                    {
                        self.queue_trigger(&source, index);
                    }
                }
            }
        }
    }

    /// One event per defender player in this declaration, with exact non-targeted subjects.
    fn defender_attack_group_triggers(&mut self, declared: &[AttackDeclaration]) {
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.state.objects[id as usize]
                        .card
                        .triggers()
                        .iter()
                        .any(|ability| {
                            ability.condition.event() == TriggerCondition::CreaturesAttackYou
                        })
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            let attackers: Vec<_> = declared
                .iter()
                .filter(|row| row.defender == Some(CombatDefender::Player(source.controller)))
                .map(|row| self.object_ref(row.card))
                .collect();
            let Some((&first, rest)) = attackers.split_first() else {
                continue;
            };
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::CreaturesAttackYou
                    && self.queue_trigger(&source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                        object: first,
                        additional_objects: rest.to_vec(),
                    }));
                }
            }
        }
    }

    /// Completed declaration; one historical event per distinct blocking creature.
    pub(super) fn opponent_blocker_triggers(&mut self, actor: Player, declared: &[CardId]) {
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id) && self.state.objects[id as usize].controller != actor
            })
            .map(|id| self.object_view(id))
            .collect();
        let mut seen = Vec::new();
        for &blocker in declared {
            if seen.contains(&blocker) {
                continue;
            }
            seen.push(blocker);
            if !self.on_battlefield(blocker)
                || !self.is_creature(blocker)
                || self.state.objects[blocker as usize].controller != actor
            {
                continue;
            }
            let stats = self.stats(blocker);
            if stats.power > 1 && stats.toughness > 1 {
                continue;
            }
            let object = self.object_ref(blocker);
            for source in &sources {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    if ability.condition.event() == TriggerCondition::OpponentCreatureBlocksSmall
                        && self.queue_trigger(source, index)
                    {
                        self.state.pending_triggers.last_mut().unwrap().event_object =
                            Some(Box::new(EventObjectView {
                                object,
                                facts: EventObjectFacts::Controller(actor),
                                conniver_creature: None,
                                subtype_lki: None,
                            }));
                    }
                }
            }
        }
    }

    /// Reuse the live attachment index once per completed declaration. No arena scan
    /// or mutable granted-ability graph; every queued packet owns its provenance.
    fn attachment_attack_triggers(&mut self, actor: Player, alone: Option<ObjectRef>) {
        self.attachment_combat_triggers(actor, alone, false);
    }
    pub(super) fn attachment_block_triggers(&mut self, actor: Player) {
        self.attachment_combat_triggers(actor, None, true);
    }
    fn attachment_combat_triggers(
        &mut self,
        actor: Player,
        alone: Option<ObjectRef>,
        blocking: bool,
    ) {
        let grants: Vec<_> = self
            .state
            .attachment_sources
            .iter()
            .filter_map(|&id| {
                let grantor = &self.state.objects[id as usize];
                let recipient = grantor.attached_to?;
                if !self.on_battlefield(id)
                    || !self.on_battlefield(recipient.card)
                    || self.object_ref(recipient.card) != recipient
                    || self.state.objects[recipient.card as usize].controller != actor
                    || if blocking {
                        !self.state.blocking.contains(&recipient.card)
                    } else {
                        !self
                            .state
                            .attackers
                            .iter()
                            .any(|a| a.card == recipient.card)
                    }
                {
                    return None;
                }
                Some((id, recipient))
            })
            .collect();
        for (id, recipient) in grants {
            let giver = self.object_view(id);
            for (index, ability) in giver.card.triggers().iter().enumerate() {
                let granted = match ability.condition.event() {
                    TriggerCondition::AttachedAttacksOrBlocks => false,
                    TriggerCondition::EquippedAttacks if !blocking => false,
                    TriggerCondition::EquippedAttacksAlone
                        if !blocking && alone == Some(recipient) =>
                    {
                        false
                    }
                    TriggerCondition::GrantedAttacks if !blocking => true,
                    TriggerCondition::SelfAttacks
                        if !blocking && ability.condition.is_granted() =>
                    {
                        true
                    }
                    _ => continue,
                };
                // Supported games have two players, owner==controller, and only
                // opposing player/planeswalker defenders. Both share this player.
                let event = Box::new(AttachmentTriggerView {
                    grantor: giver.reference,
                    granting_card: giver.card,
                    recipient,
                    defender: (!blocking).then_some(actor.other()),
                    granted,
                    origin: giver.copied_grant_origin(),
                });
                let source = if granted {
                    self.object_view(recipient.card)
                } else {
                    giver.clone()
                };
                self.queue_trigger_from(&source, index, Some(event));
            }
        }
    }

    pub(super) fn attack_triggers_for_declaration(
        &mut self,
        source: &ObjectView,
        declared_count: usize,
        first_attack: bool,
    ) {
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if match ability.condition.event() {
                TriggerCondition::SelfAttacks
                | TriggerCondition::SelfAttacksIfEquipped
                | TriggerCondition::SelfEntersOrAttacks => true,
                TriggerCondition::SelfFirstAttack => first_attack,
                TriggerCondition::SelfAttacksAlone => declared_count == 1,
                TriggerCondition::SelfAttacksWhileSaddled
                | TriggerCondition::SelfEntersOrAttacksWhileSaddled => source.saddled,
                _ => false,
            } {
                self.queue_trigger(source, index);
            }
        }
    }

    pub(super) fn prepare_triggers(&mut self) {
        if self.state.pending == Pending::Priority
            && self.state.entry_batch.is_none()
            && self.state.resolving.is_none()
            && self.state.outcome.is_none()
        {
            self.queue_state_counter_triggers();
        }
        if self.state.pending != Pending::Priority
            || self.state.entry_batch.is_some()
            || self.state.resolving.is_some()
            || self.state.outcome.is_some()
            || self.state.pending_triggers.is_empty()
        {
            return;
        }
        assert!(self.state.ordering_triggers.is_empty());
        self.expand_trigger_multiplicity();
        self.state.ordering_triggers = std::mem::take(&mut self.state.pending_triggers);
        self.continue_trigger_order();
    }

    pub(super) fn continue_trigger_order(&mut self) {
        // Controllers place their batch bottom-to-top in APNAP order. A sole
        // remaining item has no ordering choice and can be placed automatically.
        for player in [self.state.active, self.state.active.other()] {
            loop {
                let mut indices = self
                    .state
                    .ordering_triggers
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.controller == player)
                    .map(|(i, _)| i);
                let Some(index) = indices.next() else {
                    break;
                };
                if indices.next().is_some() {
                    self.state.pending = Pending::OrderTriggers { player };
                    return;
                }
                let item = self.state.ordering_triggers.remove(index);
                if self.start_announcement(item, true) {
                    return;
                }
            }
        }
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }

    pub(super) fn resolve_top(&mut self) {
        let mut item = self.state.stack.pop().unwrap();
        self.retain_departing_activation_for_triggers(&item);
        // Radiation has an intervening-if condition at trigger and resolution.
        if let StackKind::InherentRule {
            rule: SourceLessRule::Radiation,
            affected_player,
        } = item.kind
            && self.state.players[affected_player.index()]
                .player_counter(crate::cards::PlayerCounterKind::Rad)
                == 0
        {
            self.give_priority();
            return;
        }
        if let StackKind::TriggeredAbility { ability } = item.kind
            && (!self.cast_payment_condition(
                item.trigger_definition().triggers()[ability as usize].condition,
                item.source.expect("triggered ability has source").object,
                item.cast_event.as_deref(),
            ) || !self.trigger_item_guards_hold(
                &item,
                item.trigger_definition().triggers()[ability as usize].condition,
            ) || !self.tap_event_guard(&item)
                || !self.source_quantity_guard(&item)
                || !self.event_object_guard(&item))
        {
            self.give_priority();
            return;
        }
        let was_targeted = !item.targets.is_empty();
        let spec = self.target_spec(&item);
        // Preserve the whole original target set for relational legality. A departed
        // target still contributes its last controller, even though it cannot move.
        // The empty common case allocates nothing.
        let relation_illegal: Vec<_> = item
            .targets
            .iter()
            .copied()
            .filter(|bound| !self.resolution_target_relations_hold(&item, *bound, spec))
            .collect();
        // Keep type slots paired with their targets when some targets go illegal.
        let type_bindings = item.target_type_slots.as_ref().map(|slots| {
            item.targets
                .iter()
                .copied()
                .zip(slots.iter().copied())
                .collect::<Vec<_>>()
        });
        // Retain the existing allocation while borrowing the packet's source facts.
        let mut targets = std::mem::take(&mut item.targets);
        targets.retain(|bound| {
            !relation_illegal.contains(bound)
                && self.same_target_object(*bound)
                && spec
                    .group(bound.group)
                    .is_some_and(|group| self.item_target_matches(&item, group.rule, bound.target))
        });
        item.targets = targets;
        if let Some(bindings) = type_bindings {
            item.target_type_slots = Some(Box::new(
                item.targets
                    .iter()
                    .map(|target| {
                        bindings
                            .iter()
                            .find(|(bound, _)| bound == target)
                            .expect("retained target slot")
                            .1
                    })
                    .collect(),
            ));
        }
        self.retain_resolution_target_budgets(&mut item, spec);
        let valid = !was_targeted || !item.targets.is_empty();
        if !valid {
            if item.kind == StackKind::Spell {
                if item
                    .cast
                    .is_some_and(|c| matches!(c.face, CastFace::Adventure | CastFace::Omen))
                {
                    self.move_adventure_spell(&item, Zone::Graveyard);
                } else {
                    self.move_object(item.source.expect("spell has source").id, Zone::Graveyard);
                }
            }
            self.give_priority();
            return;
        }
        let program = match item.kind {
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedTokenCreation { .. }
            | StackKind::DelayedManifestDread
            | StackKind::DelayedWarpExile
            | StackKind::SiegeDefeat
            | StackKind::InherentRule { .. }
            | StackKind::ActivatedAbility { .. } => {
                item.program().expect("selected ability program").1
            }
            StackKind::Spell => {
                let source = item.source.expect("spell has source");
                match source.card.effect() {
                    Effect::Aura(_) => {
                        self.record_resolved_spell_name(&item);
                        let target = item.targets[0].object.unwrap();
                        let mut movement = self.entry_move(source.id, Zone::Battlefield, false);
                        movement.attachment = Some(target);
                        movement.cast_event = item.actual_cast_event().map(Box::new);
                        movement.cast_payment =
                            item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                        movement.cast_origin =
                            item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
                        movement.gift = item.gift_context().map(Box::new);
                        movement.warp_controller = item
                            .cast
                            .filter(|c| c.method == CastMethod::Warp)
                            .map(|_| item.controller);
                        self.stage_entries(vec![movement], None);
                        self.give_priority();
                        return;
                    }
                    Effect::Creature(..) | Effect::Permanent => {
                        if source.card.other_face().is_some() {
                            self.state.objects[source.id as usize].card = source.card;
                        }
                        self.record_resolved_spell_name(&item);
                        if item.cast.is_some_and(|c| {
                            c.offspring == Some(true)
                                || c.kicker == Some(true)
                                || c.bargain == Some(true)
                                || c.evidence == Some(true)
                                || matches!(c.gift, Some(GiftDeclaration::Promised { .. }))
                                || matches!(
                                    c.method,
                                    CastMethod::Warp | CastMethod::Sneak | CastMethod::Disguise
                                )
                                || source.card.entry_counter_rules().iter().any(|r| {
                                    matches!(
                                        r.amount,
                                        EntryCounterAmount::ColorsSpent
                                            | EntryCounterAmount::IfNotCastOrZero(_)
                                    )
                                })
                        }) || item.x.is_some()
                            || self.has_cast_entry_counters(source.object)
                            || !source.card.hand_cast_entry_counters().is_empty()
                            || !source.card.cast_entry_counters().is_empty()
                            || source.card.entry_choice().is_some()
                            || self.has_entry_counter_replacements(source.card)
                            || source.card.kind_mask() & crate::cards::BATTLE_KIND != 0
                        {
                            let mut movement = self.entry_move(
                                source.id,
                                Zone::Battlefield,
                                item.cast.is_some_and(|c| c.method == CastMethod::Sneak),
                            );
                            movement.attacking = item.cast.and_then(|c| c.sneak_defender);
                            movement.warp_controller = item
                                .cast
                                .filter(|c| c.method == CastMethod::Warp)
                                .map(|_| item.controller);
                            movement.cast_event = item.actual_cast_event().map(Box::new);
                            movement.cast_payment =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                            movement.cast_origin =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
                            movement.offspring_paid =
                                item.cast.is_some_and(|c| c.offspring == Some(true));
                            movement.kicked = item.cast.is_some_and(|c| c.kicker == Some(true));
                            movement.gift = item.gift_context().map(Box::new);
                            movement.bargained = item.cast.is_some_and(|c| c.bargain == Some(true));
                            movement.x = item.x.unwrap_or(0);
                            movement.evidence_collected =
                                item.cast.is_some_and(|c| c.evidence == Some(true));
                            self.stage_entries(vec![movement], None);
                        } else {
                            let mut event = self.move_object_raw(source.id, Zone::Battlefield);
                            event.cast_event = item.actual_cast_event().map(Box::new);
                            event.was_cast = item.cast.is_some() && !item.is_copy;
                            self.finish_zone_events(std::slice::from_ref(&event), &[]);
                        }
                        self.give_priority();
                        return;
                    }
                    Effect::Spell(..) | Effect::Modal(_) => {
                        source.card.spell_program(item.mode).unwrap().1
                    }
                    Effect::Land | Effect::Emblem | Effect::Unsupported => {
                        unreachable!("unsupported spell admitted")
                    }
                }
            }
        };
        if program
            .iter()
            .any(|i| matches!(i, Instruction::IfThisAbilityResolutionNumber { .. }))
        {
            self.record_ability_resolution(&item);
        }
        assert!(self.state.resolving.is_none());
        let reads_target_power = program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::AmassGoblinsScaled {
                    amount: AmountValue::TargetCreaturePower,
                    ..
                }
            )
        });
        let reads_target_attacking = program.iter().any(|instruction| {
            matches!(instruction, Instruction::DrawTargetControllerIfNotAttacking)
        });
        let reads_target_legendary_kind = program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::DrawIfTargetLegendaryKind(_)));
        let target_card_facts = program
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    Instruction::EnsureGoblinArmyFor(PlayerSet::TargetController)
                        | Instruction::AmassGoblinsScaled {
                            player: PlayerSet::TargetController,
                            ..
                        }
                        | Instruction::DrawIfTargetController
                        | Instruction::DrawTargetControllerIfNotAttacking
                        | Instruction::DrawIfTargetLegendaryKind(_)
                        | Instruction::LoseLifeTargetCardManaValue(_)
                        | Instruction::MayTargetController { .. }
                        | Instruction::SearchTargetController(_)
                        | Instruction::LoseLifeTargetController(_)
                        | Instruction::DamageTargetController(_)
                        | Instruction::DecreaseLeadingSpeed {
                            players: PlayerSet::TargetController,
                            ..
                        }
                )
            })
            .then(|| {
                Box::new(
                    item.targets
                        .iter()
                        .filter_map(|bound| match bound.target {
                            Target::Card(card) => Some(TargetCardFacts {
                                group: bound.group,
                                power: if reads_target_power && self.is_creature(card) {
                                    self.stats(card).power
                                } else {
                                    0
                                },
                                controller: self.state.objects[card as usize].controller,
                                legendary: reads_target_legendary_kind
                                    && ability_world::CharacteristicWorld::legendary(self, card),
                                kinds: if reads_target_legendary_kind {
                                    self.object_kinds(card, &self.state.objects[card as usize])
                                } else {
                                    0
                                },
                                mana_value: self.object_mana_value(card),
                                attacking: reads_target_attacking
                                    && self.state.attackers.iter().any(|a| a.card == card),
                            }),
                            _ => None,
                        })
                        .collect(),
                )
            });
        self.state.resolving = Some(Resolution {
            chosen_card_name: None,
            chosen_creature_type: None,
            target_counter_removal: None,
            linked_copy: None,
            cascade: None,
            target_card_facts,
            sacrificed_power: None,
            immediate_blink_return: None,
            explore: None,
            evidence: None,
            connive: None,
            proliferate: None,
            item,
            program,
            pc: 0,
            target_group: 0,
            moved_count: None,
            remembered_draws: None,
            remembered_creatures: Vec::new(),
            discard_after: None,
            optional_wheel: None,
            discards: [0; 2],
            mana_payment: None,
            forage: None,
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            milled_selection: None,
            library: None,
            selection: None,
            return_attachment: None,
            target_library: None,
            search: None,
            search_result: None,
            optional_searches: None,
            choice: None,
        });
        self.run_program();
    }

    pub(super) fn program_players(
        &self,
        set: PlayerSet,
        item: &SpellView,
    ) -> impl Iterator<Item = Player> + use<> {
        let controller = item.controller;
        let players = match set {
            PlayerSet::SourceOwner => [
                item.source().and_then(|source| {
                    self.state
                        .objects
                        .get(source.id as usize)
                        .map(|object| object.owner)
                }),
                None,
            ],
            PlayerSet::TargetController => [
                self.state.resolving.as_ref().and_then(|r| {
                    r.target_card_facts
                        .as_ref()?
                        .iter()
                        .find(|f| f.group == r.target_group)
                        .map(|f| f.controller)
                }),
                None,
            ],
            PlayerSet::EventPlayer => [
                Some(item.event_player.expect("event player must be captured")),
                None,
            ],
            PlayerSet::CastingPlayer => [
                Some(
                    item.cast_event
                        .as_ref()
                        .expect("casting player requires cast event")
                        .controller,
                ),
                None,
            ],
            PlayerSet::DamagedPlayer => [
                Some(
                    item.damage_event
                        .expect("damaged player requires a damage event")
                        .player,
                ),
                None,
            ],
            PlayerSet::DrawingPlayer => [
                Some(
                    item.draw_event
                        .expect("drawing player requires draw event")
                        .player,
                ),
                None,
            ],
            PlayerSet::Active => [Some(self.state.active), None],
            PlayerSet::Controller => [Some(controller), None],
            PlayerSet::Opponent => [Some(controller.other()), None],
            PlayerSet::Each => [Some(self.state.active), Some(self.state.active.other())],
            PlayerSet::SpeedBelow(threshold) => {
                let first = self.state.active;
                [first, first.other()].map(|player| {
                    (self.state.players[player.index()].speed < threshold).then_some(player)
                })
            }
            PlayerSet::Targeted => {
                let first = self.state.active;
                let selected = |player| {
                    item.targets
                        .iter()
                        .any(|bound| bound.target == Target::Player(player))
                };
                [
                    selected(first).then_some(first),
                    selected(first.other()).then_some(first.other()),
                ]
            }
        };
        players.into_iter().flatten()
    }

    /// Selected players own a resolution-time recipient set; creatures are not targets.
    pub(super) fn player_creature_recipients(
        &self,
        players: PlayerSet,
        item: &SpellView,
    ) -> Vec<CardId> {
        let controllers: Vec<_> = self.program_players(players, item).collect();
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                self.on_battlefield(*id as CardId)
                    && self.is_creature(*id as CardId)
                    && controllers.contains(&object.controller)
            })
            .map(|(id, _)| id as CardId)
            .collect()
    }

    /// Discard all chosen hands together, then draw in APNAP order.
    /// Entry replacements and interrupted draws suspend between stages.
    fn advance_optional_wheel(&mut self) -> bool {
        let Some(wheel) = self.state.resolving.as_ref().and_then(|r| r.optional_wheel) else {
            return false;
        };
        if !wheel.running {
            return false;
        }
        if wheel.next >= 3 {
            self.state.resolving.as_mut().unwrap().optional_wheel = None;
            return false;
        }
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .optional_wheel
            .as_mut()
            .unwrap()
            .next += 1;
        if wheel.next == 0 {
            let cause = self.state.resolving.as_ref().unwrap().item.controller;
            let cards: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, object)| {
                    (object.zone == Zone::Hand
                        && object.is_card()
                        && wheel.choices[object.owner.index()] == Some(true))
                    .then_some(id as CardId)
                })
                .collect();
            self.discard_batch_with_after(&cards, None, Some(cause), None);
        } else {
            let player = if wheel.next == 1 {
                wheel.first
            } else {
                wheel.first.other()
            };
            if wheel.choices[player.index()] == Some(true) {
                self.draw_many(player, u128::from(wheel.amount));
            }
        }
        true
    }
    pub(super) fn run_program(&mut self) {
        if self.miracle_choice_pending() {
            return;
        }
        // Casting during an effect suspends its owned program, including through
        // additional-cost entry and replacement continuations.
        if let Some(free) = &self.state.resolution_free_casts {
            if self.state.announcing.is_none() && self.state.entry_batch.is_none() {
                self.state.pending = Pending::ResolutionCasts {
                    player: free.player,
                };
            }
            return;
        }
        loop {
            // Type transitions remove combatants before the next instruction.
            self.refresh_combat_battle_types();
            // A transient threshold between instructions grants the designation
            // immediately, before a later instruction can remove a permanent.
            self.refresh_enduring_stories();
            self.refresh_graveyard_borrowing();
            if self.miracle_choice_pending() {
                return;
            }
            self.finish_cascade();
            if self.advance_replacements(false) {
                return;
            }
            if self.advance_proliferate() {
                return;
            }
            if self.state.entry_batch.is_some()
                || self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .mana_payment
                    .is_some()
            {
                return;
            }
            if self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .choice
                .as_ref()
                .is_some_and(|choice| matches!(choice.kind, EffectChoiceKind::DiscardedLand { .. }))
                && self.advance_effect_choice()
            {
                return;
            }
            if self.advance_optional_wheel() {
                continue;
            }
            if self.resume_immediate_blink() {
                continue;
            }
            if self.advance_optional_searches() {
                return;
            }
            if self.advance_return_attachment() {
                return;
            }
            if self.finish_entered_library_selection() {
                return;
            }
            if self.state.resolving.as_ref().unwrap().connive.is_some() && self.advance_connive() {
                return;
            }
            // Mandatory discards are chosen privately by their player, APNAP when
            // multiple players must discard. Hand exhaustion completes that part.
            for player in [self.state.active, self.state.active.other()] {
                let remaining = self.state.resolving.as_ref().unwrap().discards[player.index()];
                if remaining > 0 {
                    if self.hand_count(player)
                        > self.state.resolving.as_ref().unwrap().selected_discards[player.index()]
                            .len()
                    {
                        self.state.pending = Pending::EffectDiscard { player, remaining };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().discards[player.index()] = 0;
                }
            }
            // All choices precede the simultaneous discard event. Later players
            // must not see earlier players' selected hand identities.
            let selected =
                std::mem::take(&mut self.state.resolving.as_mut().unwrap().selected_discards);
            let discarded: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|player| selected[player.index()].iter().copied())
                .collect();
            let after = self.state.resolving.as_mut().unwrap().discard_after.take();
            if !discarded.is_empty() || after.is_some() {
                let cause = self.state.resolving.as_ref().unwrap().item.controller;
                self.discard_batch_with_after(&discarded, None, Some(cause), after);
                if self.state.entry_batch.is_some()
                    || self.state.resolving.as_ref().unwrap().choice.is_some()
                    || self.miracle_choice_pending()
                {
                    return;
                }
            }
            let resolution = self.state.resolving.as_mut().unwrap();
            let Some(instruction) = resolution.program.get(resolution.pc).copied() else {
                self.abandon_linked_choices();
                let item = self.state.resolving.take().unwrap().item;
                self.record_resolved_spell_name(&item);
                if item.kind == StackKind::Spell
                    && self.state.objects[item
                        .source
                        .expect("instruction requires an object source")
                        .id as usize]
                        .zone
                        == Zone::Stack
                {
                    if item.cast.is_some_and(|c| c.face == CastFace::Adventure) {
                        self.finish_adventure(&item);
                    } else if item.cast.is_some_and(|c| c.face == CastFace::Omen) {
                        self.finish_omen(&item);
                    } else {
                        self.move_adventure_spell(&item, Zone::Graveyard);
                    }
                }
                self.prune_linked_exiles();
                self.give_priority();
                return;
            };
            resolution.pc += 1;
            let mut item = resolution.item.clone();
            if !matches!(
                instruction,
                Instruction::Fight { .. }
                    | Instruction::RevealTargetHandExileUntilSourceLeaves
                    | Instruction::MayAttachControlledEquipment { .. }
                    | Instruction::AttachEquipmentByGroups { .. }
                    | Instruction::AttachEquipmentGroupsReflexive { .. }
                    | Instruction::ReturnAllTargetsToHand
                    | Instruction::CopyTargetSpell { .. }
                    | Instruction::CopyTargetAbility { .. }
                    | Instruction::DamageByGroup { .. }
                    | Instruction::LoyaltyDamage { .. }
                    | Instruction::PowerDamage { .. }
                    | Instruction::CreaturePowerDamageAny { .. }
                    | Instruction::SacrificeTarget { .. }
                    | Instruction::ScaledPowerDamage { .. }
                    | Instruction::CreateCopyTokenForPlayer { .. }
                    | Instruction::BecomeCopy(..)
                    | Instruction::GainControl {
                        controller: PlayerSet::Targeted,
                        ..
                    }
            ) {
                item.targets
                    .retain(|bound| bound.group == resolution.target_group);
            }
            match instruction {
                Instruction::OfferMiracleCast(cost) => {
                    if self.offer_miracle_cast(&item, cost) {
                        return;
                    }
                }
                Instruction::Cascade => {
                    if self.start_cascade(&item) {
                        return;
                    }
                }
                Instruction::Discover(amount) => {
                    if self.start_discover(&item, amount) {
                        return;
                    }
                }
                Instruction::PrepareSource => self.prepare_permanent(
                    item.source
                        .expect("instruction requires an object source")
                        .object,
                ),
                Instruction::PrepareControlledCreatures => {
                    let creatures: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, object)| {
                            (object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.is_creature(id as CardId))
                            .then_some(self.object_ref(id as CardId))
                        })
                        .collect();
                    for creature in creatures {
                        self.prepare_permanent(creature);
                    }
                }
                Instruction::ChooseCardName { nonland } => {
                    self.state.pending = Pending::CardName {
                        player: item.controller,
                        nonland,
                    };
                    return;
                }
                Instruction::ExileTargetNamedCards { maximum } => {
                    if self.start_named_card_search(&item, maximum) {
                        return;
                    }
                }
                Instruction::ChooseLandCardName => {
                    self.state.pending = Pending::LandCardName {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::Prepare { subject } => {
                    for id in self.keyword_subjects(&item, subject) {
                        self.prepare_permanent(self.object_ref(id));
                    }
                }
                Instruction::Unprepare { subject } => {
                    // The Aura's last attached permanent can stop being a creature
                    // before this ability resolves; it is still the same object.
                    let recipients = if subject == Subject::Attached {
                        self.attached_permanent(&item).into_iter().collect()
                    } else {
                        self.keyword_subjects(&item, subject)
                    };
                    for id in recipients {
                        self.forget_prepared(self.object_ref(id));
                    }
                }
                Instruction::PartitionLibraryPiles(maximum) => {
                    self.start_library_piles(item.controller, maximum);
                    return;
                }
                Instruction::KeepOneOfEachPermanentType => {
                    if self.start_keep_types(item.controller.other()) {
                        return;
                    }
                }
                Instruction::RememberOwnSacrifice => self.remember_own_sacrifice(),
                Instruction::RecoverAnotherPermanent => {
                    if self.recover_another_permanent(item.controller) {
                        return;
                    }
                }
                Instruction::ReturnLinkedExileToHand => self.return_linked_exile(
                    item.source
                        .expect("instruction requires an object source")
                        .object,
                ),
                Instruction::ReturnOneLinkedExileToHand => {
                    if !self
                        .linked_exile_candidates(
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                        )
                        .is_empty()
                    {
                        self.state.pending = Pending::LinkedExile {
                            player: item.controller,
                        };
                        return;
                    }
                }
                Instruction::ScheduleOpponentAttackUntilNextTurn { ability } => {
                    let mut delayed = item.clone();
                    delayed.id = self.next_stack_id();
                    delayed.kind = StackKind::TriggeredAbility { ability };
                    delayed.targets.clear();
                    delayed.crime_qualifying_target = false;
                    delayed.target_type_slots = None;
                    delayed.mode = None;
                    delayed.cast = None;
                    delayed.is_copy = false;
                    delayed.related_stack = None;
                    delayed.additional_triggers = 0;
                    self.state
                        .delayed_opponent_attack_until_next_turn
                        .get_or_insert_default()
                        .push(delayed);
                }
                Instruction::GrowDeclaredAttackers {
                    power,
                    toughness,
                    keywords,
                } => {
                    let ids: Vec<_> = item
                        .reflexive_subject
                        .as_ref()
                        .into_iter()
                        .flat_map(|subject| {
                            std::iter::once(subject.object)
                                .chain(subject.additional_objects.iter().copied())
                        })
                        .filter(|reference| {
                            self.on_battlefield(reference.card)
                                && self.object_ref(reference.card) == *reference
                        })
                        .map(|reference| reference.card)
                        .collect();
                    for &id in &ids {
                        self.state.objects[id as usize].growth += Stats {
                            power: i128::from(power),
                            toughness: i128::from(toughness),
                        };
                    }
                    self.grant_keyword_effects(
                        &ids,
                        item.source.expect("object-backed packet").object,
                        keywords,
                    );
                }
                Instruction::WeakenDeclaredAttacker(power) => {
                    if let Some(subject) = &item.reflexive_subject
                        && self.on_battlefield(subject.object.card)
                        && self.object_ref(subject.object.card) == subject.object
                    {
                        self.state.objects[subject.object.card as usize].growth += Stats {
                            power: i128::from(power),
                            toughness: 0,
                        };
                    }
                }
                Instruction::ExileOpponentLibrariesExceptBottom => {
                    let library = &self.state.players[item.controller.other().index()].library;
                    let ids = library.iter().skip(1).copied().collect::<Vec<_>>();
                    let before = self.death_trigger_sources();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    self.finish_zone_events(&events, &before);
                }
                Instruction::DelayTargetEvent { event, ability } => {
                    self.delay_target_event(&item, event, ability);
                }
                Instruction::ScheduleAttackTriggerThisTurn { ability } => {
                    let mut delayed = item.clone();
                    delayed.id = self.next_stack_id();
                    delayed.kind = StackKind::TriggeredAbility { ability };
                    delayed.targets.clear();
                    delayed.crime_qualifying_target = false;
                    delayed.target_type_slots = None;
                    delayed.mode = None;
                    delayed.cast = None;
                    delayed.is_copy = false;
                    delayed.related_stack = None;
                    delayed.additional_triggers = 0;
                    self.state
                        .delayed_attack_this_turn
                        .get_or_insert_default()
                        .push(delayed);
                }
                Instruction::ScheduleSacrificedArtifactCopy { ability } => {
                    let mut delayed = item.clone();
                    delayed.id = self.next_stack_id();
                    delayed.kind = StackKind::TriggeredAbility { ability };
                    delayed.targets.clear();
                    delayed.crime_qualifying_target = false;
                    delayed.target_type_slots = None;
                    self.state
                        .delayed_artifact_copies
                        .get_or_insert_default()
                        .push(delayed);
                }
                Instruction::CopyEnteringPermanentThenAnimate(spec) => {
                    self.copy_entering_then_animate(&item, spec);
                }
                Instruction::CreateCapturedArtifactCopy => {
                    if let Some(recipe) = item.captured_artifact_copy.as_deref() {
                        self.stage_copy_creation(item.controller, *recipe, 1);
                    }
                }
                Instruction::CopyNextExhaustAbility { ability } => {
                    self.remember_next_spell_copy(&item, ability)
                }
                Instruction::CopyNextInstantSorcery { ability } => {
                    self.remember_next_spell_copy(&item, ability)
                }
                Instruction::CopySpellsThisTurn { ability } => {
                    self.remember_next_spell_copy(&item, ability)
                }
                Instruction::ExileTargetsCopyFreeWithBudget { mana_value } => {
                    if self.exile_targets_copy_free_with_budget(&item, mana_value) {
                        return;
                    }
                }
                Instruction::CopyPaidExiledCardsFree { maximum_casts } => {
                    if self.copy_paid_exiled_cards_free(&item, maximum_casts) {
                        return;
                    }
                }
                Instruction::CopyPreparedCastSpell => {
                    if self.copy_captured_prepared_spell(&item) {
                        return;
                    }
                }
                Instruction::CopyCastSpell { .. }
                | Instruction::CopyCastSpellChosenX
                | Instruction::CopyStormSpell => {
                    if self.copy_captured_spell(&item) {
                        return;
                    }
                }
                Instruction::ChangeCastSpellTargets => {
                    if self.change_cast_spell_targets(&item) {
                        return;
                    }
                }
                Instruction::ChangeSingleStackTarget => {
                    if self.change_single_stack_target(&item) {
                        return;
                    }
                }
                Instruction::IfThisAbilityResolutionNumber { number, skip } => {
                    if self.ability_resolution_number(&item) != number {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::GiveGift => self.give_gift(&item),
                Instruction::ReturnSpellToHand => self.return_targeted_spells_to_hand(&item),
                Instruction::ProhibitCastingThisTurn => self.state.casting_prohibited = true,
                Instruction::HarnessSource => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        let rows = &mut self
                            .state
                            .roles
                            .get_or_insert_with(Default::default)
                            .harnessed;
                        if !rows.contains(
                            &item
                                .source
                                .expect("instruction requires an object source")
                                .object,
                        ) {
                            rows.push(
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                            );
                        }
                    }
                }
                Instruction::PlayerHexproofThisTurn => {
                    self.state.players[item.controller.index()].hexproof_until_end_turn = true;
                }
                Instruction::GrantPermanentsThisTurn(keywords) => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, object)| {
                            (object.zone == Zone::Battlefield
                                && object.controller == item.controller)
                                .then_some(id as CardId)
                        })
                        .collect();
                    self.grant_keyword_effects(
                        &ids,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        keywords,
                    );
                }
                Instruction::DamageWithController {
                    damage,
                    controller_damage,
                } => {
                    let source = self.program_damage_source(&item);
                    let mut events = Vec::new();
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                            && self.on_battlefield(id)
                        {
                            let controller = self.state.objects[id as usize].controller;
                            events.push(DamageEvent {
                                preventable: true,
                                source,
                                target: bound.target,
                                amount: i128::from(damage),
                                combat: false,
                            });
                            events.push(DamageEvent {
                                preventable: true,
                                source,
                                target: Target::Player(controller),
                                amount: i128::from(controller_damage),
                                combat: false,
                            });
                        }
                    }
                    self.apply_damage_batch(&events);
                }
                Instruction::IfGift { promised, skip } => {
                    if item.gift_context().is_some() != promised {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfAnotherInstantSorceryCast { skip } => {
                    let own_cast = !item.is_copy
                        && item.cast.is_some_and(|cast| cast.caster == item.controller);
                    if self.state.players[item.controller.index()].instant_sorcery_cast_this_turn
                        <= u64::from(own_cast)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfManaSpent {
                    minimum,
                    maximum,
                    skip,
                } => {
                    let amount = self
                        .item_cast_payment(&item)
                        .into_iter()
                        .map(u64::from)
                        .sum::<u64>();
                    if !minimum.is_none_or(|v| amount >= v) || !maximum.is_none_or(|v| amount <= v)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::FightSource => self.source_fight(&item),
                Instruction::FightAttached => self.attached_fight(&item),
                Instruction::IfTargetPlayerLife { life, skip } => {
                    let holds = item.targets.iter().any(|bound| {
                        matches!(bound.target, Target::Player(player)
                            if self.state.players[player.index()].life == life)
                    });
                    if !holds {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::CounterAndCreateForController { token, amount } => {
                    for bound in &item.targets {
                        if let Some((id, controller)) =
                            self.bound_stack_item(bound).map(|v| (v.id, v.controller))
                        {
                            self.counter_stack_item(id, false);
                            self.create_tokens(controller, token, u128::from(amount), false);
                        }
                    }
                }
                Instruction::CounterAndDelayManaIfSubtype { subtype, ability } => {
                    for bound in &item.targets {
                        let Some((id, event)) = self
                            .bound_stack_item(bound)
                            .map(|victim| (victim.id, victim.actual_cast_event()))
                        else {
                            continue;
                        };
                        // The payment belongs to the legal target even when countering is prohibited.
                        self.counter_stack_item(id, false);
                        if self.state.objects.iter().enumerate().any(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == item.controller
                                && self.has_subtype(id as CardId, subtype)
                        }) {
                            let mut delayed = item.clone();
                            delayed.id = self.next_stack_id();
                            delayed.kind = StackKind::TriggeredAbility { ability };
                            delayed.targets.clear();
                            delayed.crime_qualifying_target = false;
                            delayed.target_type_slots = None;
                            delayed.mode = None;
                            delayed.cast = None;
                            delayed.is_copy = false;
                            delayed.cast_event = event.map(Box::new);
                            delayed.related_stack = None;
                            self.state
                                .delayed_main_mana
                                .get_or_insert_default()
                                .push(delayed);
                        }
                    }
                }
                Instruction::IfTargetSpellManaValueAtMost { maximum, skip } => {
                    let qualifies = item.targets.iter().any(|bound| {
                        self.bound_stack_item(bound).is_some_and(|spell| {
                            spell.kind == StackKind::Spell && spell.mana_value() <= maximum
                        })
                    });
                    if !qualifies {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::CounterThenDrawIfDiscounted => {
                    for bound in &item.targets {
                        if let Some(victim) = self.bound_stack_item(bound) {
                            let draw =
                                victim.cast.map_or(0, CastInfo::mana_spent) < victim.mana_value();
                            let id = victim.id;
                            self.counter_stack_item(id, false);
                            if draw {
                                self.draw_many(item.controller, 1);
                            }
                        }
                    }
                }
                Instruction::Mobilize { token, amount } => {
                    let count = self.amount_value(
                        amount,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        item.controller,
                    );
                    if self.start_mobilize(token, count, &item) {
                        return;
                    }
                }
                Instruction::SacrificeDelayedTokens => self.sacrifice_delayed_tokens(&item),
                Instruction::ExileWarpedPermanent => self.exile_warped_permanent(&item),
                Instruction::ResolveSiegeDefeat => {
                    if self.resolve_siege_defeat(&item) {
                        return;
                    }
                }
                Instruction::Crew { .. } => self.resolve_crew(&item),
                Instruction::BecomeSaddled { .. } => self.resolve_saddle(&item),
                Instruction::MarkSaddled(subject) => self.mark_saddled_subject(&item, subject),
                Instruction::AdditionalLandPlays(amount) => {
                    let granted =
                        &mut self.state.players[item.controller.index()].extra_land_plays_granted;
                    *granted = granted
                        .checked_add(amount)
                        .capacity("land allowance exhausted");
                    let available =
                        &mut self.state.players[item.controller.index()].extra_land_plays_remaining;
                    *available = available
                        .checked_add(amount)
                        .capacity("land allowance exhausted");
                }
                Instruction::TopLibrary | Instruction::BottomLibrary => {
                    for bound in &item.targets {
                        if self.valid_library_target(&item, *bound) {
                            self.move_target_to_library(
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                bound.object.unwrap(),
                                if instruction == Instruction::TopLibrary {
                                    LibraryDestination::Top
                                } else {
                                    LibraryDestination::Bottom
                                },
                            );
                        }
                    }
                }
                Instruction::ShuffleGraveyardIntoLibrary => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| {
                            o.zone == Zone::Graveyard && o.owner == item.controller && o.is_card()
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    for id in ids {
                        self.move_object(id, Zone::Library);
                    }
                    self.forget_library_top(item.controller);
                    self.shuffle_library_cards(item.controller);
                }
                Instruction::DoubleTargetPower => {
                    let affected: Vec<_> = item
                        .targets
                        .iter()
                        .filter(|b| self.same_target_object(**b))
                        .filter_map(|b| match b.target {
                            Target::Card(id) if self.is_creature(id) => {
                                Some((id, self.stats(id).power))
                            }
                            _ => None,
                        })
                        .collect();
                    for (id, power) in affected {
                        self.state.objects[id as usize].growth += Stats {
                            power,
                            toughness: 0,
                        };
                    }
                }
                Instruction::DrawAndProtectCounterCreatures => {
                    // Freeze exact recipients before drawing, then grant to those incarnations.
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == item.controller
                                && o.counters.plus_one > 0
                                && self.is_creature(*id as CardId)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    self.draw_many(item.controller, ids.len() as u128);
                    if !self.defer_draw_protection(
                        ids.clone(),
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                    ) {
                        self.grant_keyword_effects(
                            &ids,
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                            Keywords::INDESTRUCTIBLE,
                        );
                    }
                }
                Instruction::ShuffleTargetIntoLibrary => self.shuffle_target_into_library(&item),
                Instruction::ShuffleSourceAndTargetIntoLibraries => {
                    self.shuffle_source_and_target_into_libraries(&item)
                }
                Instruction::MultiplyTargetStats { factor } => {
                    self.multiply_target_stats(&item, factor)
                }
                Instruction::OwnerLibrary => {
                    if self.start_target_library(&item, false) {
                        return;
                    }
                }
                Instruction::OwnerLibraryTopOrBottomDamage(amount) => {
                    if self.start_target_library_damage(&item, amount) {
                        return;
                    }
                }
                Instruction::OwnerLibrarySecondFromTop => {
                    if self.start_target_library(&item, true) {
                        return;
                    }
                }
                Instruction::OwnerLibraryNthFromTop(n) => self.put_targets_nth_from_top(&item, n),
                Instruction::PutSourceThirdFromTop => self.put_source_third_from_top(&item),
                Instruction::ChooseManaValueParityDestroyOthers => {
                    self.state.pending = Pending::ManaValueParity {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::Forage { skip } => {
                    self.start_resolution_forage(skip);
                    return;
                }
                Instruction::CreateTokensReflexiveWithSubjects {
                    token,
                    amount,
                    ability,
                } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::ReflexiveCreated {
                            item: Box::new(item.clone()),
                            ability,
                        },
                    );
                }
                Instruction::AttachReflexiveSubjects => {
                    if self.attach_reflexive_subjects(&item) {
                        return;
                    }
                }
                Instruction::GrantEquippedAttackers(keywords) => {
                    let recipients: Vec<_> = self
                        .state
                        .attackers
                        .iter()
                        .filter_map(|a| {
                            let id = a.card;
                            (self.on_battlefield(id)
                                && self.is_creature(id)
                                && self
                                    .source_equipment_count(self.object_ref(id))
                                    .unwrap_or(0)
                                    > 0)
                            .then_some(id)
                        })
                        .collect();
                    self.grant_keyword_effects(
                        &recipients,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        keywords,
                    );
                }
                Instruction::CreateTokensReflexive {
                    token,
                    amount,
                    ability,
                } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::Reflexive {
                            item: Box::new(item.clone()),
                            ability,
                        },
                    );
                }
                Instruction::SacrificeSourceIfAttachedCounters { minimum } => {
                    if self.attached_subject(&item).is_some_and(|id| {
                        self.state.objects[id as usize].counters.plus_one >= minimum
                    }) && self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .controller
                            == item.controller
                    {
                        self.sacrifice_batch(&[(
                            item.controller,
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                        )]);
                    }
                }
                Instruction::PutSourceCounterReflexive {
                    kind,
                    amount,
                    ability,
                } => {
                    self.put_source_counter_reflexive(&item, kind, amount, ability);
                }
                Instruction::ReturnTargetAttackingWithCounter(counter) => {
                    if self.start_target_attacking_return(&item, counter) {
                        return;
                    }
                }
                Instruction::MayTapSourceReflexive { .. } => {
                    let skip_to = resolution.pc;
                    if self.exact_source_can_tap(&item) {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                }
                Instruction::Reflexive { ability } => {
                    self.queue_reflexive(&item, ability, None);
                }
                Instruction::ReflexiveSacrificedPower { ability } => {
                    if item.sacrificed_power.is_some() {
                        self.queue_reflexive(&item, ability, None);
                    }
                }
                Instruction::Modal(_)
                | Instruction::ModalOnce(_)
                | Instruction::ModalOncePerTurn(_)
                | Instruction::ModalUpToX { .. } => {
                    unreachable!("unselected modal ability executed")
                }
                Instruction::DestroyOthersIfSourcePowerTwenty => {
                    let live = self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object;
                    let power = if live {
                        self.stats(
                            item.source
                                .expect("instruction requires an object source")
                                .object
                                .card,
                        )
                        .power
                    } else {
                        item.source
                            .expect("instruction requires an object source")
                            .facts
                            .power
                    };
                    if power == 20 {
                        let mut ids: Vec<_> = (0..self.state.objects.len())
                            .map(|id| id as CardId)
                            .filter(|&id| {
                                self.on_battlefield(id)
                                    && self.is_creature(id)
                                    && self.object_ref(id)
                                        != item
                                            .source
                                            .expect("instruction requires an object source")
                                            .object
                                    && !self.keywords(id).contains(Keywords::INDESTRUCTIBLE)
                            })
                            .collect();
                        ids.retain(|&id| !self.effect_destruction_prevented(id));
                        let sources = if ids.is_empty() {
                            Vec::new()
                        } else {
                            self.death_trigger_sources()
                        };
                        let events = self.move_objects_raw(&ids, Zone::Graveyard);
                        self.finish_zone_events(&events, &sources);
                    }
                }
                Instruction::Explore(subject) => {
                    if self.start_explore(&item, subject) {
                        return;
                    }
                }
                Instruction::ManifestDread => {
                    if self.start_manifest_dread(item.controller) {
                        return;
                    }
                }
                Instruction::ManifestDreadWhenTargetDies => self.register_manifest_death(&item),
                Instruction::DestroyAndCreateForController { token, amount } => {
                    let targets = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|b| self.same_target_object(*b))
                        .collect::<Vec<_>>();
                    self.destroy_and_create(targets, token, amount);
                }
                Instruction::CollectEvidence { threshold, skip } => {
                    if self.start_resolution_evidence(threshold, skip, false) {
                        return;
                    }
                }
                Instruction::EvidenceWard(threshold) => {
                    if self.start_resolution_evidence(threshold, 0, true) {
                        return;
                    }
                }
                Instruction::IfEvidence { skip } => {
                    if !(item.evidence_collected
                        || item.cast.is_some_and(|c| c.evidence == Some(true)))
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::SacrificeGreatestPower => {
                    if self.start_greatest_power_choice(&item) {
                        return;
                    }
                }
                Instruction::IfBargained { skip } => {
                    if !item.cast.is_some_and(|c| c.bargain == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfTeamwork { skip } => {
                    if !item.cast.is_some_and(|c| c.teamwork == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfBlighted { skip } => {
                    if !item.cast.is_some_and(|c| c.blight == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfKicked { skip } => {
                    if !item.cast.is_some_and(|c| c.kicker == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }

                Instruction::IfEventObjectSubtype { subtype, skip } => {
                    if !self.event_object_has_subtype(&item, subtype) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfSubject {
                    subject,
                    filter,
                    skip,
                } => {
                    if !self.subject_condition(&item, subject, filter) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::If { guard, skip } => {
                    if !self.step_guard(
                        guard,
                        item.controller,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        item.source_tapped,
                    ) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::Skip(skip) => self.state.resolving.as_mut().unwrap().pc += skip,
                Instruction::AddCombatRed(amount) => {
                    self.add_combat_red(item.controller, u128::from(amount))
                }
                Instruction::AddCombatRedScaled(value) => {
                    let amount = self.item_amount(value, &item);
                    self.add_combat_red(item.controller, amount);
                }
                Instruction::AddPoison(set, amount) => {
                    let mut poison = [0; 2];
                    for player in self.program_players(set, &item) {
                        poison[player.index()] = u64::from(amount);
                    }
                    self.effect_counters_with_poison(
                        Vec::new(),
                        poison,
                        item.controller,
                        replacements::ResumeMode::Program,
                    );
                }
                Instruction::ExileAttackingCreatures(players) => {
                    self.exile_attacking_creatures(players, &item)
                }
                Instruction::BeginOptionalWheel(amount) => {
                    self.state.resolving.as_mut().unwrap().optional_wheel = Some(OptionalWheel {
                        first: self.state.active,
                        choices: [None; 2],
                        amount,
                        next: 0,
                        running: false,
                    });
                }
                Instruction::AskOptionalWheel { first, skip } => {
                    let wheel = resolution.optional_wheel.unwrap();
                    let player = if first {
                        wheel.first
                    } else {
                        wheel.first.other()
                    };
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    self.state.pending = Pending::EffectOptional { player, skip_to };
                    return;
                }
                Instruction::RecordOptionalWheel { first, accept } => {
                    let wheel = self
                        .state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .optional_wheel
                        .as_mut()
                        .unwrap();
                    let player = if first {
                        wheel.first
                    } else {
                        wheel.first.other()
                    };
                    wheel.choices[player.index()] = Some(accept);
                }
                Instruction::ExecuteOptionalWheel => {
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .optional_wheel
                        .as_mut()
                        .unwrap()
                        .running = true;
                }
                Instruction::MayPlayer { player, skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    let mut players = self.program_players(player, &item);
                    if let Some(player) = players.next() {
                        assert!(players.next().is_none(), "single-player optional effect");
                        self.state.pending = Pending::EffectOptional { player, skip_to };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::MayTargetController { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if let Some(player) = self.resolution_target_controller() {
                        self.state.pending = Pending::EffectOptional { player, skip_to };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::ExileAndGainLifeEqualPower => {
                    for bound in &item.targets {
                        if !self.same_target_object(*bound) {
                            continue;
                        }
                        let Target::Card(id) = bound.target else {
                            continue;
                        };
                        if !self.on_battlefield(id) {
                            continue;
                        }
                        let controller = self.state.objects[id as usize].controller;
                        let power = self.stats(id).power.max(0);
                        self.move_object(id, Zone::Exile);
                        self.gain_life(controller, power);
                    }
                }
                Instruction::MayOpponent { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    self.state.pending = Pending::EffectOptional {
                        player: item.controller.other(),
                        skip_to,
                    };
                    return;
                }
                Instruction::MayPayOpponentLife { amount, skip }
                | Instruction::MayPayControllerLife { amount, skip } => {
                    let player = if matches!(instruction, Instruction::MayPayControllerLife { .. })
                    {
                        item.controller
                    } else {
                        item.controller.other()
                    };
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if amount != 0 && self.state.players[player.index()].life < i128::from(amount) {
                        self.state.resolving.as_mut().unwrap().pc = skip_to;
                    } else {
                        self.state.pending = Pending::EffectOptional { player, skip_to };
                        return;
                    }
                }
                Instruction::ShuffleSourceIntoOwnerLibrary => {
                    self.shuffle_source_into_owner_library(&item);
                }
                Instruction::BlinkImmediately { counter, tapped } => {
                    self.blink_immediately(&item, counter, tapped)
                }
                Instruction::IfCastFromGraveyard { skip } => {
                    if item.is_copy || !item.cast.is_some_and(|cast| cast.origin == Zone::Graveyard)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfNotCastFromHand { skip } => {
                    if !item.is_copy && item.cast.is_some_and(|cast| cast.origin == Zone::Hand) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfCastFromNonHand { skip } => {
                    if item.is_copy || !item.cast.is_some_and(|cast| cast.origin != Zone::Hand) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::ExileSmallReturnUntilEndStep => {
                    self.exile_small_return_until_end_step(&item);
                }
                Instruction::BlinkUntilEndStep { under_owner } => {
                    self.blink_until_end_step(&item, under_owner)
                }
                Instruction::ReturnDeathCard(kind) => self.return_death_card(&item, kind),
                Instruction::ReturnEventCardToHand => self.return_event_card_to_hand(&item),
                Instruction::ReturnEventCardToBattlefieldTapped => {
                    self.resolve_delayed_return(&item)
                }
                Instruction::ResolveDelayedReturn => self.resolve_delayed_return(&item),
                Instruction::PreventTargetActivationsWhileTapped => {
                    self.prevent_target_activations_until_untapped(&item);
                }
                Instruction::ReturnSourceFromFirstPublicZoneToHand => {
                    self.return_source_from_first_public_zone_to_hand(&item);
                }
                Instruction::ReturnSourceToHand(origin) => {
                    let zone = match origin {
                        TargetZone::Battlefield => Zone::Battlefield,
                        TargetZone::Graveyard => Zone::Graveyard,
                    };
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == zone
                    {
                        self.move_object(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Hand,
                        );
                    }
                }
                Instruction::PutSourceOnLibraryTop { origin } => {
                    let zone = match origin {
                        TargetZone::Battlefield => Zone::Battlefield,
                        TargetZone::Graveyard => Zone::Graveyard,
                    };
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == zone
                    {
                        let owner = self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .owner;
                        let event = self.move_object(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Library,
                        );
                        if event.moved && event.destination == Zone::Library {
                            self.remember_library_top(owner, event.after);
                        }
                    }
                }
                Instruction::ReturnSourceBlankWithHaste => {
                    self.return_source_blank_with_haste(&item)
                }
                Instruction::ReturnSourceWithCounterAndEquipment(counter) => {
                    self.return_source_with_equipment(&item, counter);
                }
                Instruction::ReturnSourceAuraAttached => {
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == Zone::Graveyard
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .owner
                            == item.controller
                        && let Some(target) = item.targets.first().and_then(|bound| bound.object)
                        && self.aura_can_enchant(
                            item.source
                                .expect("instruction requires an object source")
                                .card,
                            item.controller,
                            target,
                        )
                    {
                        let mut movement = self.entry_move(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Battlefield,
                            false,
                        );
                        movement.controller = item.controller;
                        movement.attachment = Some(target);
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::ReturnOpponentsBattlefieldGraveyardCardsAsFood => {
                    self.return_opponents_battlefield_graveyard_as_food(&item);
                }
                Instruction::ReturnSourceAsEnchantment => {
                    let returned = item
                        .source_move
                        .as_ref()
                        .filter(|event| {
                            event.before == item.source.expect("source").object
                                && event.origin == Zone::Battlefield
                                && event.destination == Zone::Graveyard
                        })
                        .map(|event| event.after);
                    if let Some(returned) = returned
                        && self.object_ref(returned.card) == returned
                        && self.state.objects[returned.card as usize].zone == Zone::Graveyard
                        && !self.state.objects[returned.card as usize].is_token()
                    {
                        let mut movement = self.entry_move(returned.card, Zone::Battlefield, false);
                        movement.controller = self.state.objects[returned.card as usize].owner;
                        movement.enchantment_return = true;
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::ReturnSourceAsArtifact => {
                    let returned = item
                        .source_move
                        .as_ref()
                        .filter(|event| {
                            event.before
                                == item
                                    .source
                                    .expect("instruction requires an object source")
                                    .object
                                && event.origin == Zone::Battlefield
                                && event.destination == Zone::Graveyard
                        })
                        .map(|event| event.after);
                    if let Some(returned) = returned
                        && self.object_ref(returned.card) == returned
                        && self.state.objects[returned.card as usize].zone == Zone::Graveyard
                        && !self.state.objects[returned.card as usize].is_token()
                    {
                        let mut movement = self.entry_move(returned.card, Zone::Battlefield, false);
                        movement.controller = item.controller;
                        movement.artifact_return = true;
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::ReturnSourceAndTargetsTapped => {
                    let mut moves: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => Some(self.entry_move(id, Zone::Battlefield, true)),
                            _ => None,
                        })
                        .collect();
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == Zone::Graveyard
                    {
                        moves.push(
                            self.entry_move(
                                item.source
                                    .expect("instruction requires an object source")
                                    .id,
                                Zone::Battlefield,
                                true,
                            ),
                        );
                    }
                    self.stage_entries(moves, None);
                }
                Instruction::PutHandSourceAttacking => {
                    if let Some(source) = item.source
                        && self.object_ref(source.id) == source.object
                        && self.state.objects[source.id as usize].zone == Zone::Hand
                    {
                        let mut movement = self.entry_move(source.id, Zone::Battlefield, true);
                        movement.controller = item.controller;
                        movement.attacking = item.ninjutsu_defender;
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::ReturnSourceToBattlefield { tapped } => {
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == Zone::Graveyard
                    {
                        let movement = self.entry_move(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Battlefield,
                            tapped,
                        );
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::ReturnSourceToBattlefieldWithCounter { tapped, counter } => {
                    if self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == Zone::Graveyard
                    {
                        let mut movement = self.entry_move(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Battlefield,
                            tapped,
                        );
                        movement.controller = item.controller;
                        movement.effect_counter = Some((counter, 1));
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::MayPayWardPoison { amount: _, skip } => {
                    let link = item.related_stack.expect("ward stack association");
                    let pc = self.state.resolving.as_ref().unwrap().pc;
                    if self.player_counters_prohibited(
                        link.controller,
                        crate::cards::PlayerCounterKind::Poison,
                    ) {
                        // A prohibition makes the cost impossible before replacements.
                        self.state.resolving.as_mut().unwrap().pc = pc + skip;
                    } else {
                        self.state.pending = Pending::EffectOptional {
                            player: link.controller,
                            skip_to: pc + skip,
                        };
                        return;
                    }
                }
                Instruction::PayWardPoison(amount) => {
                    let payer = item
                        .related_stack
                        .expect("ward stack association")
                        .controller;
                    self.pay_poison_counter_cost(payer, amount);
                }
                Instruction::CounterRelated => {
                    let link = item.related_stack.expect("ward stack association");
                    self.counter_stack_item(link.id, false);
                }
                Instruction::MoveBattlefield { filters, exile } => {
                    self.move_battlefield_group(&item, filters, exile);
                }
                Instruction::ExchangeControlTargets => self.exchange_control_targets(&item),
                Instruction::GainControlUntilEndTurn => self.gain_control_until_end_turn(&item),
                Instruction::GainControlConditionalVillainManaValue => {
                    self.gain_control_conditional_villain_mana_value(&item)
                }
                Instruction::GainControl {
                    subject,
                    controller,
                } => {
                    self.gain_control(&item, subject, controller);
                }
                Instruction::ChooseGraveyardReturn {
                    maximum,
                    total_mana_value,
                } => {
                    if self.start_graveyard_return_selection(&item, maximum, total_mana_value) {
                        return;
                    }
                }
                Instruction::ReturnGraveyardManaValue { kinds, value } => {
                    self.return_graveyard_mana_value(&item, kinds, value);
                }
                Instruction::DestroyManaValue { kinds, maximum } => {
                    let maximum =
                        u32::try_from(self.item_amount(maximum, &item)).unwrap_or(u32::MAX);
                    self.move_battlefield_group(
                        &item,
                        &[BattlefieldFilter {
                            kinds,
                            relation: -1,
                            nonland: false,
                            tapped: None,
                            without: Keywords::NONE,
                            subtype: None,
                            max_mana: maximum,
                        }],
                        false,
                    );
                }
                Instruction::ReturnOwnedGraveyardNonlandPermanents => {
                    self.return_owned_graveyard_nonland_permanents(&item);
                }
                Instruction::ReturnGraveyardCreatures {
                    players,
                    maximum_mana_value,
                } => self.return_graveyard_creatures(&item, players, maximum_mana_value),
                Instruction::DestroyTargetNameGroup => self.destroy_target_name_group(&item),
                Instruction::ReturnBattlefield {
                    filters,
                    attacking,
                    players,
                } => self.return_battlefield_group(&item, filters, attacking, players),
                Instruction::DamageBattlefield {
                    filters,
                    players,
                    amount,
                } => {
                    self.damage_battlefield_group(&item, filters, players, amount);
                }
                Instruction::DamageCreaturesExceptControlledTokens(amount) => {
                    self.damage_creatures_except_controlled_tokens(&item, amount);
                }
                Instruction::DamageBattlefieldExceptSubtype {
                    filters,
                    excluded_subtype,
                    players,
                    amount,
                } => self.damage_battlefield_group_excluding(
                    &item,
                    filters,
                    players,
                    amount,
                    Some(excluded_subtype),
                ),
                Instruction::DrawMoved | Instruction::GainLifeMoved => {
                    let count = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .moved_count
                        .expect("result instruction requires a preceding group move");
                    if instruction == Instruction::DrawMoved {
                        self.draw_many(item.controller, u128::from(count));
                    } else {
                        self.gain_life(item.controller, i128::from(count));
                    }
                }
                Instruction::Connive { subject, amount } => {
                    self.start_connive(&item, subject, amount)
                }
                Instruction::GrantGraveyardCast(kind) => self.grant_graveyard_cast(&item, kind),
                Instruction::GrantCreatureGraveyardCast => {
                    self.grant_creature_graveyard_cast(item.controller)
                }
                Instruction::SolveSource => self.solve_case(
                    item.source
                        .expect("instruction requires an object source")
                        .object,
                ),
                Instruction::ReduceArtifactSpellCostsThisTurn(amount) => {
                    self.grant_artifact_spell_reduction(item.controller, amount)
                }
                Instruction::NextRedOrGreenCreatureFreeThisTurn => {
                    self.grant_next_creature_free(item.controller)
                }
                Instruction::GrantMilledPlay => self.grant_milled_play(item.controller),
                Instruction::ExileDiscardedWithStash => self.exile_discarded_with_stash(&item),
                Instruction::ExileHandDraw => self.exile_hand_draw(&item),
                Instruction::GrantExiledResultPlay { next_turn } => {
                    self.grant_exiled_result_play(item.controller, next_turn);
                }
                Instruction::MayExileDiscardedPlay { .. } => {
                    let skip_to = resolution.pc;
                    if self.discarded_play_object(&item).is_some() {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                }
                Instruction::CastGraveyard {
                    kinds,
                    exile_instants_sorceries,
                } => {
                    self.offer_paid_graveyard_cast(&item, kinds, exile_instants_sorceries);
                    return;
                }
                Instruction::CastTargetedGraveyardFree => {
                    if self.offer_targeted_graveyard_free_cast(&item) {
                        return;
                    }
                }
                Instruction::ExileTargetLibraryTopCastFree {
                    amount,
                    maximum_casts,
                } => {
                    if self.exile_target_top_cast_free(&item, amount, maximum_casts) {
                        return;
                    }
                }
                Instruction::ExileTopMayCastPaid => {
                    // Reuse the owned resolution count to retain the committed result
                    // across the cast announcement, mana payment and snapshots.
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    if self.exile_top_offer_paid_cast(item.controller) {
                        return;
                    }
                }
                Instruction::IfOfferedCastCommitted { skip } => {
                    if self.state.resolving.as_ref().unwrap().moved_count == Some(1) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::AllowJaceLoyaltyAnytime => {
                    self.state.players[item.controller.index()].jace_loyalty_anytime = true;
                }
                Instruction::AddMountainManaTrigger => {
                    let bonus =
                        &mut self.state.players[item.controller.index()].mountain_mana_bonus;
                    *bonus = bonus
                        .checked_add(1)
                        .capacity("mountain mana trigger count exhausted");
                }
                Instruction::ExileEachLibraryTopCastFree => {
                    self.exile_each_top_cast_free(item.controller);
                    return;
                }
                Instruction::CounterRelatedUnlessPayLife(amount) => {
                    if self.start_resolution_life(&item, amount) {
                        return;
                    }
                }
                Instruction::CounterUnlessPayScaled { amount, exile } => {
                    let generic = u32::try_from(self.item_amount(amount, &item))
                        .capacity("counter payment exhausted");
                    if self.start_resolution_payment(
                        &item,
                        ManaCost {
                            colored: [0; 6],
                            generic,
                        },
                        CounterSubject::Targets,
                        exile,
                        0,
                    ) {
                        return;
                    }
                }
                Instruction::CounterUnlessPay {
                    cost,
                    subject,
                    exile,
                    unpaid_skip,
                } => {
                    if self.start_resolution_payment(&item, cost, subject, exile, unpaid_skip) {
                        return;
                    }
                }
                Instruction::DrawDamageAmount => {
                    self.draw_many(
                        item.controller,
                        item.damage_event
                            .expect("damage-sized effect requires event")
                            .amount,
                    );
                }
                Instruction::DiscardThenReturnLand { tapped } => {
                    self.state.resolving.as_mut().unwrap().discards[item.controller.index()] = 1;
                    self.state.resolving.as_mut().unwrap().discard_after =
                        Some(discard_events::DiscardAfter::ReturnLandFromGraveyard {
                            player: item.controller,
                            tapped,
                        });
                }
                Instruction::DiscardThenPutLand {
                    players,
                    amount,
                    tapped,
                } => {
                    for player in self.program_players(players, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] =
                            usize::from(amount);
                    }
                    self.state.resolving.as_mut().unwrap().discard_after =
                        Some(discard_events::DiscardAfter::ChooseLand {
                            player: item.controller,
                            tapped,
                        });
                }
                Instruction::DiscardHand(players) => {
                    for player in self.program_players(players, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] =
                            self.hand_count(player);
                    }
                }
                Instruction::DiscardHandRememberCount(players) => {
                    let participants: Vec<_> = self.program_players(players, &item).collect();
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.zone == Zone::Hand
                                && object.is_card()
                                && participants.contains(&object.owner)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    // Reset before dispatch: a linked entry may pause the batch, and
                    // an empty hand must overwrite any earlier instruction result.
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    self.discard_batch_with_after(
                        &ids,
                        None,
                        Some(item.controller),
                        Some(discard_events::DiscardAfter::RememberActual),
                    );
                }
                Instruction::PutPlayerCountersWithSelf {
                    players,
                    kind,
                    amount,
                    effects,
                } => {
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    let mut counters = [[0; 5]; 2];
                    let mut owned = Vec::new();
                    for player in players {
                        counters[player.index()][kind.index()] = amount;
                        for (index, effect) in effects.iter().enumerate() {
                            owned.push(replacements::SelfPlayerCounterReplacement {
                                origin_stack_id: item.id,
                                effect: u16::try_from(index).expect("self replacement index"),
                                player,
                                kind,
                                factor: effect.factor,
                                add: effect.add,
                                redirect_player: effect
                                    .redirect_to_opponent
                                    .then(|| player.other()),
                                redirect_kind: effect.redirect_kind,
                                split: effect.split,
                            });
                        }
                    }
                    self.effect_mixed_player_counters_with_self(
                        counters,
                        item.controller,
                        item.id,
                        owned,
                        replacements::ResumeMode::Program,
                    );
                }
                Instruction::DecreaseLeadingSpeed { players, minimum } => {
                    // Freeze qualifying recipients before changing either player's designation.
                    let players: Vec<_> = self
                        .program_players(players, &item)
                        .filter(|player| {
                            self.state.players[player.index()].speed
                                > self.state.players[player.other().index()].speed
                        })
                        .collect();
                    for player in players {
                        let speed = &mut self.state.players[player.index()].speed;
                        if *speed > minimum {
                            *speed -= 1;
                        }
                    }
                }
                Instruction::IncreaseSpeed => {
                    let StackKind::InherentRule {
                        rule: SourceLessRule::Speed,
                        ..
                    } = item.kind
                    else {
                        unreachable!()
                    };
                    // “Your speed” belongs to this packet's controller, including a copy.
                    let speed = &mut self.state.players[item.controller.index()].speed;
                    if *speed < 4 {
                        *speed += 1;
                    }
                }
                Instruction::ResolveRadiation => {
                    let StackKind::InherentRule {
                        rule: SourceLessRule::Radiation,
                        affected_player: player,
                    } = item.kind
                    else {
                        unreachable!()
                    };
                    let count = self.state.players[player.index()]
                        .player_counter(crate::cards::PlayerCounterKind::Rad);
                    let nonland = self.mill_radiation(player, count);
                    self.lose_life(player, i128::from(nonland));
                    if let Some(counters) = self.state.players[player.index()]
                        .other_player_counters
                        .as_mut()
                    {
                        let rad = &mut counters[crate::cards::PlayerCounterKind::Rad.index() - 1];
                        *rad = rad.saturating_sub(nonland);
                    }
                }
                Instruction::MillLifeEvent => {
                    let player = item.life_event.expect("saved life event").player;
                    self.mill_batch(&[player], MillAmount::LifeEvent, item.controller);
                }
                Instruction::DrawForGraveyardsAtLeast(minimum) => {
                    let count = [Player::P0, Player::P1]
                        .into_iter()
                        .filter(|&player| {
                            self.state
                                .objects
                                .iter()
                                .filter(|object| {
                                    object.owner == player
                                        && object.zone == Zone::Graveyard
                                        && object.is_card()
                                })
                                .count()
                                >= usize::from(minimum)
                        })
                        .count();
                    self.draw_many(item.controller, count as u128);
                }
                Instruction::Mill(players, amount) => {
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    self.mill_batch(&players, amount, item.controller);
                }
                Instruction::MillThenReflexive { amount, ability } => {
                    let milled = self.mill_batch(
                        &[item.controller],
                        MillAmount::Fixed(amount),
                        item.controller,
                    );
                    if milled == usize::from(amount) {
                        self.queue_reflexive(&item, ability, None);
                    }
                }
                Instruction::MayMill(amount) => {
                    // CR701.17b forbids choosing to mill more than the library.
                    let enough_cards =
                        self.library_card_count(item.controller) >= usize::from(amount);
                    let resolution = self.state.resolving.as_mut().unwrap();
                    if !enough_cards {
                        resolution.pc += 1;
                    } else {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to: resolution.pc + 1,
                        };
                        return;
                    }
                }
                Instruction::SelectMilled(spec) => {
                    if self.start_milled_selection(spec) {
                        return;
                    }
                }
                Instruction::CollectGraveyard => {
                    self.state.resolving.as_mut().unwrap().milled = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.zone == Zone::Graveyard
                                && object.owner == item.controller
                                && object.is_card()
                        })
                        .map(|(id, object)| ObjectRef {
                            card: id as CardId,
                            generation: object.generation,
                        })
                        .collect();
                }
                Instruction::CastLibraryTopUpToAttackingPower(amount) => {
                    self.cast_library_by_attacking_power(&item, amount);
                    return;
                }
                Instruction::BottomLookedLibraryRandom => self.finish_library_cast_look(),
                Instruction::ControlTargetPlayerNextTurn => {
                    for player in self
                        .program_players(PlayerSet::Targeted, &item)
                        .collect::<Vec<_>>()
                    {
                        self.schedule_player_control(player, item.controller);
                    }
                }
                Instruction::ExileAllCreaturesRemember => self.exile_all_creatures_remember(),
                Instruction::ExileAllCreaturesCount => self.exile_all_creatures_count(),
                Instruction::DeployHandCreaturesEach => {
                    self.start_hand_deployment();
                    return;
                }
                Instruction::ReturnExiledResultToHands => self.return_exiled_result_to_hands(),
                Instruction::ExileDamagedLibraryUntilNonland => {
                    self.exile_damaged_library_until_nonland(&item)
                }
                Instruction::MayCounterSourceOtherwiseCast => {
                    if self.offer_source_counter_or_cast(&item) {
                        return;
                    }
                }
                Instruction::LookTargetHandMayDiscardThenDraw => {
                    if self.start_private_hand_discard_draw(&item) {
                        return;
                    }
                }
                Instruction::RevealTargetHandSubsetDiscard(amount) => {
                    if self.start_hand_subset_discard(&item, amount) {
                        return;
                    }
                }
                Instruction::RevealTargetHandExileLinked => {
                    if self.start_revealed_hand_exile_linked(&item) {
                        return;
                    }
                }
                Instruction::RevealTargetHandExileUntilSourceLeaves => {
                    if self.start_revealed_hand_exile(&item) {
                        return;
                    }
                }
                Instruction::RevealHandDiscard { excluded_kinds } => {
                    let Some(victim) = self.program_players(PlayerSet::Targeted, &item).next()
                    else {
                        continue;
                    };
                    let hand: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.zone == Zone::Hand && object.owner == victim && object.is_card()
                        })
                        .map(|(id, _)| self.object_ref(id as CardId))
                        .collect();
                    for reference in &hand {
                        self.remember_public_hand(*reference);
                    }
                    if hand.iter().any(|reference| {
                        self.state.objects[reference.card as usize].card.kind_mask()
                            & excluded_kinds
                            == 0
                    }) {
                        self.state.pending = Pending::RevealedDiscard {
                            player: item.controller,
                            victim,
                            excluded_kinds,
                        };
                        return;
                    }
                }
                Instruction::DestroyLoseManaValue => {
                    let loss: u64 = item
                        .targets
                        .iter()
                        .filter_map(|bound| {
                            if !self.same_target_object(*bound) {
                                return None;
                            }
                            if let Target::Card(id) = bound.target {
                                Some(self.object_mana_value(id))
                            } else {
                                None
                            }
                        })
                        .sum();
                    let destroy: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|bound| match bound.target {
                            Target::Card(id) => !self.effect_destruction_prevented(id),
                            _ => false,
                        })
                        .collect();
                    self.move_target_batch(&destroy, Zone::Graveyard);
                    self.lose_life(item.controller, i128::from(loss));
                }
                Instruction::RecoverMilled(filter) => {
                    if self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .milled
                        .iter()
                        .any(|&object| self.can_recover_milled(item.controller, filter, object))
                    {
                        self.state.pending = Pending::RecoverMilled {
                            player: item.controller,
                            filter,
                        };
                        return;
                    }
                }
                Instruction::TapAttached => {
                    if let Some(id) = self.attached_permanent(&item) {
                        self.tap_objects(&[id]);
                    }
                }
                Instruction::AttachEquipmentToAttached { group } => {
                    if self.attach_selected_equipment_to_attached(&item, group) {
                        return;
                    }
                }
                Instruction::MayAttachControlledEquipment {
                    recipient,
                    required_subtype,
                } => {
                    if self.may_attach_controlled_equipment(&item, recipient, required_subtype) {
                        return;
                    }
                }
                Instruction::AttachEquipmentGroupsReflexive {
                    equipment,
                    recipient,
                    ability,
                } => {
                    if self.attach_groups_reflexive(&item, equipment, recipient, ability) {
                        return;
                    }
                }
                Instruction::AttachEquipmentByGroups {
                    equipment,
                    recipient,
                } => {
                    let first = item
                        .targets
                        .iter()
                        .find(|bound| bound.group == equipment)
                        .and_then(|bound| bound.object);
                    let second = item
                        .targets
                        .iter()
                        .find(|bound| bound.group == recipient)
                        .and_then(|bound| bound.object);
                    if let (Some(first), Some(second)) = (first, second) {
                        self.attach_equipment(first, second);
                    }
                }
                Instruction::GainClassLevel(level) => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        let object = &mut self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .object
                            .card
                            as usize];
                        let changed = object.class_level != level;
                        object.class_level = level;
                        if changed {
                            let source = self.object_view(item.source.expect("class source").id);
                            for (index, ability) in source.card.triggers().iter().enumerate() {
                                if ability.condition.event()
                                    == TriggerCondition::BecomesClassLevel(level)
                                {
                                    self.queue_trigger(&source, index);
                                }
                            }
                        }
                    }
                }
                Instruction::AttachSource => {
                    if let Some(target) = item.targets.iter().find_map(|bound| bound.object) {
                        self.attach_equipment(
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                            target,
                        );
                    }
                }
                Instruction::EnsureJaceToken => self.ensure_jace_token(item.controller),
                Instruction::EmpowerJace(amount) => {
                    if self.empower_jace(&item, u64::from(amount)) {
                        return;
                    }
                }
                Instruction::EmpowerJaceScaled(amount) => {
                    let amount = u64::try_from(self.item_amount(amount, &item))
                        .capacity("empower counter amount exhausted");
                    if self.empower_jace(&item, amount) {
                        return;
                    }
                }
                Instruction::EnsureGoblinArmy | Instruction::EnsureGoblinArmyFor(_) => {
                    let players = match instruction {
                        Instruction::EnsureGoblinArmyFor(players) => players,
                        _ => PlayerSet::Controller,
                    };
                    let recipients: Vec<_> = self.program_players(players, &item).collect();
                    assert!(recipients.len() <= 1, "one amassing player per instruction");
                    if let Some(player) = recipients.first().copied() {
                        self.ensure_goblin_army(player);
                    }
                }
                Instruction::AmassGoblins(amount) | Instruction::AmassGoblinsThenAttach(amount) => {
                    if self.amass_goblins_for(&item, item.controller, u64::from(amount)) {
                        return;
                    }
                }
                Instruction::AmassGoblinsScaled { player, amount } => {
                    let amount = u64::try_from(self.item_amount(amount, &item))
                        .capacity("amass counter amount exhausted");
                    let recipients: Vec<_> = self.program_players(player, &item).collect();
                    assert!(recipients.len() <= 1, "one amassing player per instruction");
                    if let Some(player) = recipients.first().copied()
                        && self.amass_goblins_for(&item, player, amount)
                    {
                        return;
                    }
                }
                Instruction::DrawIfTargetLegendaryKind(kinds) => {
                    let matches = self.state.resolving.as_ref().is_some_and(|r| {
                        r.target_card_facts.as_ref().is_some_and(|facts| {
                            facts.iter().any(|f| {
                                f.group == r.target_group && f.legendary && f.kinds & kinds != 0
                            })
                        })
                    });
                    if matches {
                        self.draw_many(item.controller, 1);
                    }
                }
                Instruction::DrawTargetControllerIfNotAttacking => {
                    let player = self.state.resolving.as_ref().and_then(|r| {
                        r.target_card_facts
                            .as_ref()?
                            .iter()
                            .find(|f| f.group == r.target_group && !f.attacking)
                            .map(|f| f.controller)
                    });
                    if let Some(player) = player {
                        self.draw_many(player, 1);
                    }
                }
                Instruction::DrawIfTargetController => {
                    if self
                        .program_players(PlayerSet::TargetController, &item)
                        .any(|player| player == item.controller)
                    {
                        self.draw_many(item.controller, 1);
                    }
                }
                Instruction::ChooseGrantKeywords { subject, choices } => {
                    let recipients: Vec<_> = self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .map(|id| self.object_ref(id))
                        .collect();
                    if !recipients.is_empty() {
                        assert!(!choices.is_empty() && u16::try_from(choices.len()).is_ok());
                        self.state.pending = Pending::KeywordChoice {
                            player: item.controller,
                            recipients: Box::new(recipients),
                            choices,
                        };
                        return;
                    }
                }
                Instruction::CreateTokenChoice(tokens) => {
                    self.state.pending = Pending::TokenChoice {
                        player: item.controller,
                        tokens,
                    };
                    return;
                }
                Instruction::ExileHandCardCopy => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::HandExileCopy,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::TargetStatsUntilNextTurn { power, toughness } => {
                    self.target_stats_until_next_turn(&item, power, toughness);
                }
                Instruction::CreateTokenIfControllerSacrificed(token) => {
                    if self.state.resolving.as_ref().unwrap().moved_count == Some(1) {
                        self.create_tokens(item.controller, token, 1, false);
                    }
                }
                Instruction::ChooseObjects {
                    kind,
                    players,
                    filter,
                    amount,
                    optional_skip,
                } => {
                    if self.start_effect_choice(&item, kind, players, filter, amount, optional_skip)
                    {
                        return;
                    }
                }
                Instruction::CreateNonlegendaryCopyTokens { subject, amount } => {
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        recipe.overrides = Some(CopyOverrides {
                            legendary: Some(false),
                            ..recipe.overrides.unwrap_or(CopyOverrides::NONE)
                        });
                        self.stage_copy_creation(item.controller, recipe, u128::from(amount));
                    }
                }
                Instruction::CopyTargetPlayersCreaturesWithConditionalSacrifice => {
                    self.copy_target_players_creatures_with_conditional_sacrifice(&item);
                }
                Instruction::CopyControlledNontokenCreaturesNonlegendary => {
                    self.copy_controlled_nontoken_creatures_nonlegendary(item.controller);
                }
                Instruction::CopyOtherControlledNonlands => {
                    self.copy_other_controlled_nonlands(&item);
                }
                Instruction::BecomeSourceLandUntilNextTurn => {
                    self.source_land_until_next_turn(&item)
                }
                Instruction::CreateCopyWithDelayedSacrifice => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_temporary_copy_creation(&item, recipe);
                    }
                }
                Instruction::CreateTemporaryArtifactCopy => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_temporary_copy_creation(
                            &item,
                            recipe.with_exceptions(CopyExceptions::ARTIFACT),
                        );
                    }
                }
                Instruction::BecomeCopy(spec) => self.resolve_in_place_copy(&item, spec),
                Instruction::CopyChosenExiledTriggerCreature => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::CopyExiledTriggerCreature,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::IgnoreLegendRuleThisTurn => {
                    self.state.players[item.controller.index()].legend_rule_ignored_this_turn =
                        true;
                }
                Instruction::CreateCopyTokenForPlayer {
                    subject,
                    amount,
                    player,
                } => {
                    if let Some(controller) = self.program_players(player, &item).next()
                        && let Some(recipe) = self.recipe_for_subject(&item, subject)
                    {
                        self.stage_copy_creation(controller, recipe, u128::from(amount));
                    }
                }
                Instruction::PutPermanentCountersScaled {
                    filters,
                    kind,
                    amount,
                } => {
                    self.put_permanent_counters_scaled(&item, filters, kind, amount);
                }
                Instruction::CreateCopyToken { subject, amount }
                | Instruction::CreateCopyTokenWithExceptions {
                    subject, amount, ..
                } => {
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        if let Instruction::CreateCopyTokenWithExceptions { exceptions, .. } =
                            instruction
                        {
                            recipe = recipe.with_exceptions(exceptions);
                        }
                        self.stage_copy_creation(item.controller, recipe, u128::from(amount));
                    }
                }
                Instruction::CreateCounteredCopyToken {
                    subject,
                    exceptions,
                    kind,
                    amount,
                } => {
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        recipe = recipe.with_exceptions(exceptions);
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe,
                            1,
                            replacements::TokenCompletion::Counters { kind, amount },
                        );
                    }
                }
                Instruction::SacrificeSourceIfNoPlaneswalker => {
                    if !self.controls_planeswalker(item.controller) {
                        self.sacrifice_exact_source(&item);
                    }
                }
                Instruction::SacrificeSource => self.sacrifice_exact_source(&item),
                Instruction::SacrificeSourceOrSkip { skip } => {
                    if !self.try_sacrifice_exact_source(&item) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::SacrificeSourceReflexive { ability } => {
                    if self.try_sacrifice_exact_source(&item) {
                        // Departure refreshes the saved packet's LKI, including
                        // keywords gained after the historical threshold event.
                        let sacrificed = self.state.resolving.as_ref().unwrap().item.clone();
                        self.queue_reflexive(&sacrificed, ability, None);
                    }
                }
                Instruction::ExileAndCopyNightmare => self.exile_and_copy_nightmare(&item),
                Instruction::ExileOtherNightmares => self.exile_other_nightmares(item.controller),
                Instruction::DamageDivided { .. } => self.damage_divided(&item),
                Instruction::DistributeCounters { kind, .. }
                | Instruction::DistributeCountersX { kind } => {
                    self.distribute_counters(&item, kind)
                }
                Instruction::DamageByGroup { first, other } => {
                    self.damage_by_group(&item, first, other)
                }
                Instruction::DamageForExcess(amount) => self.damage_for_excess(&item, amount),
                Instruction::IfNoExcessDamage { skip } => {
                    if self.tracked_excess_damage() == 0 {
                        self.clear_tracked_excess_damage();
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::AmassExcessDamage => {
                    let amount = self.take_tracked_excess_damage();
                    if self.amass_goblins_for(&item, item.controller, amount) {
                        return;
                    }
                }
                Instruction::EmpowerJaceExcessDamage => {
                    let amount = self.take_tracked_excess_damage();
                    if self.empower_jace(&item, amount) {
                        return;
                    }
                }
                Instruction::DamageForSubtypeDestruction { amount, subtype } => {
                    self.damage_for_subtype_destruction(&item, amount, subtype);
                }
                Instruction::DestroyDamagedSubtype => self.destroy_damaged_subtype(&item),
                Instruction::CreateExcessTokens(token) => self.create_excess_tokens(&item, token),
                Instruction::ExileExcessPlay { next_turn } => {
                    self.exile_excess_play(&item, next_turn)
                }
                Instruction::CopyTargetSpell {
                    group,
                    retarget,
                    exceptions,
                } => {
                    if self.copy_target_spell(&item, group, retarget, exceptions) {
                        return;
                    }
                }
                Instruction::CopyTargetAbility { group } => {
                    if self.copy_target_ability(&item, group) {
                        return;
                    }
                }
                Instruction::CopyCapturedActivatedAbility => {
                    if self.copy_captured_activated_ability(&item) {
                        return;
                    }
                }
                Instruction::FlipCoinsSkipTurns { coins } => {
                    self.flip_coins_skip_turns(&item, coins)
                }
                Instruction::LoseHalfLife(players) => {
                    for player in self.program_players(players, &item) {
                        // CR107.1b: a negative calculated loss is zero.
                        self.lose_life(player, self.state.players[player.index()].life.max(0) / 2);
                    }
                }
                Instruction::ChooseHalf { kind } => {
                    if self.start_half_choice(&item, kind) {
                        return;
                    }
                }
                Instruction::DiscardUpToDraw { maximum } => {
                    if self.start_discard_up_to_draw(&item, maximum) {
                        return;
                    }
                }
                Instruction::OpponentSacrificeIfAbleElseDiscard { any } => {
                    if self.opponent_sacrifice_else_discard(&item, any) {
                        return;
                    }
                }
                Instruction::DestroyTargetsAtMostManaValue(amount) => {
                    let maximum = self.item_amount(amount, &item);
                    let targets: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|bound| {
                            self.same_target_object(*bound)
                                && matches!(bound.target, Target::Card(id)
                                if u128::from(self.object_mana_value(id)) <= maximum)
                        })
                        .collect();
                    self.destroy_targets_outcome(&targets);
                }
                Instruction::DestroyTargetsThenSourceCountersIfOwnCreature { kind, amount } => {
                    let events = self.destroy_targets_outcome(&item.targets);
                    if events.iter().any(|event| {
                        event.moved
                            && event.before.zone == Zone::Battlefield
                            && event.before.kinds & 1 != 0
                            && event.before.controller == item.controller
                    }) && let Some(source) = item.source
                        && self.on_battlefield(source.id)
                        && self.object_ref(source.id) == source.object
                    {
                        self.effect_counters(
                            vec![(source.object, Counters::of(kind, u64::from(amount)))],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::DiscardUpToOneDraw => {
                    if self.start_discard_up_to_one_draw(&item) {
                        return;
                    }
                }
                Instruction::DiscardAnyDraw {
                    players,
                    bonus,
                    required_all,
                } => {
                    if self.start_discard_any_draw(&item, players, bonus, required_all) {
                        return;
                    }
                }
                Instruction::DamageCannotBePreventedThisTurn => {
                    self.state.damage_prevention_prohibited = true;
                }
                Instruction::AdditionalCombat => self.add_combat(),
                Instruction::AdditionalCombatsAfterCurrentMain(count) => {
                    self.add_combats_after_current_main(count)
                }
                Instruction::ScheduleBeginCombatThisTurn { ability } => {
                    self.schedule_begin_combat_this_turn(&item, ability)
                }
                Instruction::UntapCreaturesThatAttackedThisTurn => {
                    self.untap_creatures_that_attacked_this_turn()
                }
                Instruction::ExtraTurn {
                    power_up_prohibited,
                } => self.add_extra_turn(item.controller, power_up_prohibited),
                Instruction::TransformSource => self.transform_source(&item),
                Instruction::RequireBlocks(filter) => self.require_blocks(&item, filter),
                Instruction::ExileDamagedIfDiesThisTurn
                | Instruction::ActivationX
                | Instruction::SourceCounterCost { .. }
                | Instruction::WaterbendCost(_)
                | Instruction::ExertCost => (),
                Instruction::RemoveCreatureCounterThenDraw => {
                    if self.start_resolution_counter_payment(&item) {
                        return;
                    }
                }
                Instruction::DestroyCombatDamagedPlayersPermanents => {
                    self.destroy_combat_damaged_players_permanents(&item)
                }
                Instruction::MayRemoveAnySourceCounterReflexive(_) => {
                    if let Some(source) = item.source
                        && self.on_battlefield(source.object.card)
                        && self.object_ref(source.object.card) == source.object
                        && self.counter_amounts(source.object.card).total() > 0
                    {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .target_counter_removal = Some(Box::new(TargetCounterRemovalView {
                            target: source.object,
                            maximum: 1,
                            selected: Counters::default(),
                        }));
                        self.state.pending = Pending::TargetCounterRemoval {
                            player: item.controller,
                        };
                        return;
                    }
                }
                Instruction::RemoveUpToTargetCounters(maximum) => {
                    let target = item.targets.iter().copied().find_map(|bound| {
                        if !self.same_target_object(bound) {
                            return None;
                        }
                        match bound.target {
                            Target::Card(id) if self.on_battlefield(id) => {
                                Some(self.object_ref(id))
                            }
                            _ => None,
                        }
                    });
                    if let Some(target) = target
                        && maximum != 0
                        && self.counter_amounts(target.card).total() != 0
                    {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .target_counter_removal = Some(Box::new(TargetCounterRemovalView {
                            target,
                            maximum,
                            selected: Counters::default(),
                        }));
                        self.state.pending = Pending::TargetCounterRemoval {
                            player: item.controller,
                        };
                        return;
                    }
                }
                Instruction::PutSourceCountersOnTargets => {
                    let mut counters = Counters::default();
                    for kind in CounterKind::ALL {
                        counters.add(
                            kind,
                            u64::try_from(
                                self.item_amount(AmountValue::SourceCounterQuantity(kind), &item),
                            )
                            .capacity("counter transfer quantity exhausted"),
                        );
                    }
                    let changes = self
                        .keyword_subjects(&item, Subject::Targets)
                        .into_iter()
                        .map(|id| (self.object_ref(id), counters.clone()))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::CreateTokensWithSourceCounters { token, amount } => {
                    self.create_tokens_with_source_counters(&item, token, amount);
                }
                Instruction::CreateTokensWithCounters {
                    token,
                    amount,
                    kind,
                    counters,
                } => {
                    self.create_tokens_with_counters(&item, token, amount, kind, counters);
                }
                Instruction::CreateTokensWithGraveyardGroups {
                    token,
                    amount,
                    group,
                } => {
                    let completion = if !item.is_copy
                        && item.cast.is_some_and(|cast| cast.origin == Zone::Graveyard)
                    {
                        replacements::TokenCompletion::GraveyardGroups(group)
                    } else {
                        replacements::TokenCompletion::None
                    };
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        completion,
                    );
                }
                Instruction::SelectGroup(group) => {
                    self.state.resolving.as_mut().unwrap().target_group = group;
                }
                Instruction::CreaturePowerDamageAny { source, recipient } => {
                    let source = item.targets.iter().find_map(|bound| {
                        (bound.group == source && self.same_target_object(*bound))
                            .then_some(bound.target)
                            .and_then(|target| match target {
                                Target::Card(id)
                                    if self.on_battlefield(id) && self.is_creature(id) =>
                                {
                                    Some(id)
                                }
                                _ => None,
                            })
                    });
                    let recipient = item
                        .targets
                        .iter()
                        .find(|bound| bound.group == recipient && self.same_target_object(**bound));
                    if let (Some(source), Some(recipient)) = (source, recipient) {
                        let damage = DamageEvent {
                            preventable: true,
                            source: self.damage_source(source),
                            target: recipient.target,
                            amount: self.stats(source).power.max(0),
                            combat: false,
                        };
                        self.apply_damage_batch(&[damage]);
                    }
                }
                Instruction::SacrificeTarget { group } => {
                    let id = item.targets.iter().find_map(|bound| {
                        (bound.group == group && self.same_target_object(*bound))
                            .then_some(bound.target)
                            .and_then(|target| match target {
                                Target::Card(id)
                                    if self.on_battlefield(id)
                                        && self.state.objects[id as usize].controller
                                            == item.controller =>
                                {
                                    Some(id)
                                }
                                _ => None,
                            })
                    });
                    if let Some(id) = id {
                        self.sacrifice_batch(&[(item.controller, id)]);
                    }
                }
                Instruction::Fight { source, recipient }
                | Instruction::PowerDamage { source, recipient } => {
                    self.creature_power_damage(
                        &item,
                        source,
                        recipient,
                        matches!(instruction, Instruction::Fight { .. }),
                        1,
                    );
                }
                Instruction::LoyaltyDamage { source, recipient } => {
                    self.planeswalker_loyalty_damage(&item, source, recipient);
                }
                Instruction::ScaledPowerDamage {
                    source,
                    recipient,
                    factor,
                } => {
                    self.creature_power_damage(&item, source, recipient, false, factor);
                }
                Instruction::AddColorlessScaled(amount) => {
                    let amount = u32::try_from(self.item_amount(amount, &item))
                        .capacity("mana representation exhausted");
                    let mana = [0, 0, 0, 0, 0, amount];
                    let quality = self.item_mana_quality(&item, ManaProduction::Fixed(mana));
                    if quality.is_some() {
                        self.add_quality_mana(item.controller, mana, quality);
                    } else {
                        self.add_mana(item.controller, mana);
                    }
                }
                Instruction::AddManaScaled { color, amount } => {
                    let amount = u32::try_from(self.item_amount(amount, &item))
                        .capacity("mana representation exhausted");
                    let mut mana = [0; 6];
                    mana[color.index()] = amount;
                    let quality = self.item_mana_quality(&item, ManaProduction::Fixed(mana));
                    if quality.is_some() {
                        self.add_quality_mana(item.controller, mana, quality);
                    } else {
                        self.add_mana(item.controller, mana);
                    }
                }
                Instruction::IfSacrificedVehicle { skip } => {
                    if !item
                        .paid_additional_cost
                        .as_ref()
                        .is_some_and(|paid| paid.sacrificed_vehicle)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::AddManaChosenScaled { players, amount } => {
                    let amount = u32::try_from(self.item_amount(amount, &item))
                        .capacity("chosen mana amount exhausted");
                    let mut recipients = [false; 2];
                    for player in self.program_players(players, &item) {
                        recipients[player.index()] = true;
                    }
                    if recipients.iter().any(|&recipient| recipient) {
                        self.state.pending = Pending::EffectMana {
                            player: item.controller,
                            production: ManaProduction::any_color(amount),
                            recipients,
                        };
                        return;
                    }
                }
                Instruction::AddMana(production) => {
                    let quality = self.item_mana_quality(&item, production).map(|quality| {
                        quality.with_definition(
                            item.source
                                .expect("instruction requires an object source")
                                .card,
                        )
                    });
                    match production.base().in_world(
                        self,
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) {
                        ManaProduction::ChosenColor
                        | ManaProduction::MonocoloredPermanentColors
                        | ManaProduction::DistinctCreaturePowers
                        | ManaProduction::CountSubtype { .. }
                        | ManaProduction::SourcePower { .. }
                        | ManaProduction::TapSourcePowerChoice { .. }
                        | ManaProduction::Tagged { .. } => unreachable!("resolved production"),
                        ManaProduction::Fixed(mana) => {
                            self.add_quality_mana(item.controller, mana, quality)
                        }
                        production @ (ManaProduction::Choice { .. }
                        | ManaProduction::Combination { .. }) => {
                            self.state.pending = Pending::EffectMana {
                                player: item.controller,
                                production,
                                recipients: std::array::from_fn(|i| i == item.controller.index()),
                            };
                            if quality.is_some() {
                                self.state.mana_lots.get_or_insert_default().effect_quality =
                                    quality;
                            }
                            return;
                        }
                    }
                }
                Instruction::EventObjectDamagePlayers(players, amount) => {
                    self.event_object_damage_players(&item, players, amount);
                }
                Instruction::DamageEventObjectController(amount) => {
                    self.damage_event_object_controller(&item, amount);
                }
                Instruction::AnimateSource(spec) => self.animate_source(&item, spec),
                Instruction::AnimateSourceColored(spec, colors) => {
                    self.animate_source_colored(&item, spec, colors)
                }
                Instruction::AnimateSourceWithAbilityIfNoncreature { spec, kind } => {
                    self.animate_source_with_ability_if_noncreature(&item, spec, kind)
                }
                Instruction::AnimateTargets(spec) => self.animate_targets(&item, spec),
                Instruction::AnimateSubject { subject, spec } => {
                    self.animate_subject(&item, subject, spec)
                }
                Instruction::AnimateNoncreatureTargets(spec) => {
                    self.animate_noncreature_targets(&item, spec)
                }
                Instruction::AddArtifactToTargets => self.add_artifact_to_targets(&item),
                Instruction::ChangeSourceCreatureForm(spec) => self.change_source_form(
                    &item,
                    spec,
                    characteristic_layers::AnimationTypeMode::PreserveCardTypes,
                ),
                Instruction::SetCreatureForm {
                    types,
                    colors,
                    base,
                } => self.set_creature_form(&item, types, colors, base),
                Instruction::MoveCountersFromSource { kind, amount } => {
                    self.move_counters_from_source(&item, kind, amount)
                }
                Instruction::AnimateSourceIfNoncreature { spec, ability } => {
                    self.animate_source_if_noncreature(&item, spec, ability)
                }
                Instruction::DestroyLandAndSearch => self.destroy_land_and_search(&item),
                Instruction::SearchLibraryAndGraveyardWithXCounters {
                    filter,
                    counter,
                    haste_at,
                } => {
                    self.start_zone_search(&item, filter, counter, haste_at);
                    return;
                }
                Instruction::SearchLibraryAndOrGraveyardToHand(filter) => {
                    self.start_zone_hand_search(&item, filter);
                    return;
                }
                Instruction::SearchHandAndOrLibrary(filter) => {
                    self.start_hand_library_search(item.controller, filter);
                    return;
                }
                Instruction::Search(spec)
                | Instruction::SearchScaled { spec, .. }
                | Instruction::SearchDamageReceived(spec)
                | Instruction::SearchTargetController(spec)
                | Instruction::SearchTargetPlayer(spec)
                | Instruction::SearchTargetPlayerMovedCount(spec)
                | Instruction::SearchRequired(spec)
                | Instruction::SearchDifferentNames(spec)
                | Instruction::SearchDifferentNamesChosenX(spec) => {
                    let mut spec = spec;
                    if let Instruction::SearchScaled { maximum, .. } = instruction {
                        spec.maximum =
                            u64::try_from(self.item_amount(maximum, &item)).unwrap_or(u64::MAX);
                    }
                    if matches!(instruction, Instruction::SearchDamageReceived(_)) {
                        spec.maximum = u64::try_from(
                            item.damage_received.as_ref().expect("damage event").amount,
                        )
                        .unwrap_or(u64::MAX);
                    }
                    if matches!(instruction, Instruction::SearchDifferentNamesChosenX(_)) {
                        spec.maximum = u64::from(item.x.unwrap_or(0));
                    }
                    if matches!(instruction, Instruction::SearchTargetPlayerMovedCount(_)) {
                        spec.maximum = self
                            .state
                            .resolving
                            .as_ref()
                            .unwrap()
                            .moved_count
                            .unwrap_or(0);
                    }
                    let player = if matches!(instruction, Instruction::SearchTargetController(_)) {
                        let Some(player) = self.resolution_target_controller() else {
                            continue;
                        };
                        player
                    } else if matches!(
                        instruction,
                        Instruction::SearchTargetPlayer(_)
                            | Instruction::SearchTargetPlayerMovedCount(_)
                    ) {
                        let Some(player) = self.program_players(PlayerSet::Targeted, &item).next()
                        else {
                            continue;
                        };
                        player
                    } else {
                        item.controller
                    };
                    let minimum = if matches!(instruction, Instruction::SearchRequired(_)) {
                        assert_eq!(
                            spec.filter,
                            crate::SearchFilter::Any,
                            "required searches must not require finding a hidden quality"
                        );
                        spec.maximum.min(
                            u64::try_from(self.library_card_count(player)).expect("library length"),
                        )
                    } else {
                        0
                    };
                    self.state.resolving.as_mut().unwrap().search =
                        Some(Box::new(search::SearchChoice {
                            spec,
                            minimum,
                            different_names: matches!(
                                instruction,
                                Instruction::SearchDifferentNames(_)
                                    | Instruction::SearchDifferentNamesChosenX(_)
                            ),
                            selected: Vec::new(),
                            battlefield: None,
                            destination_pending: false,
                        }));
                    self.state.pending = Pending::EffectSearch { player };
                    return;
                }
                Instruction::SearchSacrificedManaValue { kinds, increment } => {
                    let Some(value) = item.sacrificed_mana_value.as_deref().copied() else {
                        continue;
                    };
                    let value = value
                        .checked_add(u64::from(increment))
                        .expect("sacrifice search mana value");
                    self.state.resolving.as_mut().unwrap().search =
                        Some(Box::new(search::SearchChoice {
                            spec: SearchSpec {
                                filter: crate::SearchFilter::KindsManaValue {
                                    kinds,
                                    minimum: value,
                                    maximum: value,
                                },
                                maximum: 1,
                                destination: crate::SearchDestination::Battlefield,
                                tapped: false,
                                reveal: false,
                            },
                            minimum: 0,
                            different_names: false,
                            selected: Vec::new(),
                            battlefield: None,
                            destination_pending: false,
                        }));
                    self.state.pending = Pending::EffectSearch {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::CaptureEntryResult => {
                    self.state.resolving.as_mut().unwrap().search_result =
                        Some(Box::new(search::SearchResult {
                            capture_next_entry: true,
                            ..search::SearchResult::default()
                        }));
                }
                Instruction::AnimateEntryResultIfSubtype { subtype, spec } => {
                    if let Some(recipient) = self.captured_entry_result()
                        && self.has_subtype(recipient.card, subtype)
                    {
                        self.change_object_form(
                            recipient,
                            spec,
                            characteristic_layers::AnimationTypeMode::AddCreature,
                        );
                    }
                }
                Instruction::PutEntryResultCountersIfKinds {
                    kinds,
                    kind,
                    amount,
                } => {
                    if let Some(recipient) = self.captured_entry_result()
                        && self.current_kinds(recipient.card) & kinds != 0
                    {
                        self.effect_counters(
                            vec![(recipient, Counters::of(kind, u64::from(amount)))],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::GrantEntryResultKeywords {
                    keywords,
                    permanent,
                } => {
                    if let (Some(recipient), Some(source)) =
                        (self.captured_entry_result(), item.source)
                    {
                        self.grant_keyword_effects(&[recipient.card], source.object, keywords);
                        if permanent && keywords != Keywords::NONE {
                            self.state
                                .roles
                                .as_mut()
                                .unwrap()
                                .keyword_effects
                                .last_mut()
                                .unwrap()
                                .expires_turn = None;
                        }
                    }
                }
                Instruction::ScheduleEntryResultExile { own_end_step } => {
                    if item.source.is_some()
                        && let Some(recipient) = self.captured_entry_result()
                    {
                        self.register_delayed_return_exile(&item, recipient);
                        if let Some(rows) = self.state.delayed_returns.as_mut() {
                            rows.last_mut().unwrap().view.own_end_step = own_end_step;
                        }
                    }
                }
                Instruction::ReturnGraveyardWithDelayedSacrifice(filter) => {
                    self.return_graveyard_with_delayed_sacrifice(&item, filter);
                }
                Instruction::AnimatePermanentGroup { filters, spec } => {
                    self.animate_permanent_group(&item, filters, spec);
                }
                Instruction::CaptureSearchResult => {
                    self.state.resolving.as_mut().unwrap().search_result =
                        Some(Box::new(search::SearchResult::default()));
                }
                Instruction::BeholdToUntapSearchResult(subtype) => {
                    if self.start_behold_search_result(&item, subtype) {
                        return;
                    }
                }
                Instruction::UntapSearchResultIfLandsAtLeast(minimum) => {
                    let lands = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.current_kinds(*id as CardId) & 2 != 0
                        })
                        .count();
                    let entered = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .search_result
                        .as_ref()
                        .and_then(|result| result.entered);
                    if lands >= minimum as usize
                        && let Some(land) = entered
                        && self.object_ref(land.card) == land
                        && self.on_battlefield(land.card)
                    {
                        self.untap_objects(&[land.card], untap_rules::UntapCause::Effect);
                    }
                    self.state.resolving.as_mut().unwrap().search_result = None;
                }
                Instruction::RevealTopDrawIfArtifact => self.reveal_top_draw_if_artifact(&item),
                Instruction::RevealTopMoveIfPermanent => self.reveal_top_move_if_permanent(&item),
                Instruction::ExileSourceReturnTransformed { counter } => {
                    self.exile_source_return_transformed(&item, counter)
                }
                Instruction::ReturnCraftedSourceTransformed => {
                    self.return_crafted_source_transformed(&item)
                }
                Instruction::ExileOpponentTopFaceDownPlayX => {
                    self.exile_opponent_top_face_down_play_x(&item)
                }
                Instruction::ExileLibraryTopFaceDownPlay {
                    library,
                    beneficiary,
                    amount,
                    spend_any_mana,
                } => {
                    self.exile_library_top_face_down_play(
                        &item,
                        library,
                        beneficiary,
                        amount,
                        spend_any_mana,
                    );
                }
                Instruction::ExileTopFaceDownPlay { amount, subtype } => {
                    self.exile_top_face_down_play(&item, amount, subtype)
                }
                Instruction::ExileTopPlayBoostSourceIfSubtype {
                    subtype,
                    power,
                    toughness,
                } => {
                    self.exile_top_play_boost_source(&item, subtype, power, toughness);
                }
                Instruction::ExileTopChoosePlay { amount, next_turn } => {
                    if self.exile_top_choose_play(item.controller, amount, next_turn) {
                        return;
                    }
                }
                Instruction::RandomLinkedExileToBattlefield => self
                    .random_linked_exile_to_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        item.controller,
                    ),
                Instruction::ExileTopPlayScaled { amount, next_turn } => {
                    let count = self.item_amount(amount, &item);
                    self.exile_top_play(item.controller, count, next_turn);
                }
                Instruction::ExileTopLibrary { players, amount } => {
                    let count = self.item_amount(amount, &item);
                    let ids = self
                        .program_players(players, &item)
                        .flat_map(|player| exile::top_library_cards(self, player, count))
                        .collect::<Vec<_>>();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::ExileTopPlayUntilNextEndStep(amount) => {
                    self.exile_top_play_until_next_end_step(item.controller, amount);
                }
                Instruction::ExileTopPlay { amount, next_turn } => {
                    self.exile_top_play(item.controller, u128::from(amount), next_turn);
                }
                Instruction::ExileTargetLibraryPlayLife { amount } => {
                    let count = self.item_amount(amount, &item);
                    for player in self.program_players(PlayerSet::Targeted, &item) {
                        self.exile_top_play_life(player, item.controller, count);
                    }
                }
                Instruction::ExileTargetLibraryPlay { amount } => {
                    for player in self.program_players(PlayerSet::Targeted, &item) {
                        self.exile_top_play_from(
                            player,
                            item.controller,
                            u128::from(amount),
                            false,
                        );
                    }
                }
                Instruction::UpToTargets(_) => (), // Bound before announcing targets.
                Instruction::IfAmountParity { value, odd, skip } => {
                    if (self.item_amount(value, &item) % 2 != 0) != odd {
                        let resolution = self.state.resolving.as_mut().unwrap();
                        let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                        assert!(skip_to <= resolution.program.len());
                        resolution.pc = skip_to;
                    }
                }
                Instruction::IfAmountAtLeast {
                    value,
                    minimum,
                    skip,
                } => {
                    if self.item_amount(value, &item) < minimum {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::UntapLands(maximum) => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::UntapLands { maximum },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        0,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::RevealUntilLandToGraveyard(players) => {
                    for player in self.program_players(players, &item).collect::<Vec<_>>() {
                        let library = &self.state.players[player.index()].library;
                        let cards: Vec<_> = library
                            .iter()
                            .rev()
                            .copied()
                            .filter(|&id| self.state.objects[id as usize].is_card())
                            .collect();
                        let amount = cards
                            .iter()
                            .position(|&id| self.state.objects[id as usize].card.is_land())
                            .map_or(cards.len(), |n| n + 1);
                        let ids = cards.into_iter().take(amount).collect::<Vec<_>>();
                        let events = self.move_objects_raw(&ids, Zone::Graveyard);
                        self.finish_zone_events(&events, &[]);
                    }
                }
                Instruction::NoMaximumHandSize => {
                    self.grant_no_maximum_hand_size(item.controller);
                }
                Instruction::Paradigm { ability } => self.resolve_paradigm(&item, ability),
                Instruction::ExileUntilManaValueCastFree(minimum) => {
                    self.exile_until_mana_value_cast_free(item.controller, minimum);
                    return;
                }
                Instruction::CastParadigmCopy => {
                    self.cast_paradigm_copy(&item);
                    return;
                }
                Instruction::ExileResolvingSpell => {
                    if item.kind == StackKind::Spell
                        && self.object_ref(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                        ) == item
                            .source
                            .expect("instruction requires an object source")
                            .object
                        && self.state.objects[item
                            .source
                            .expect("instruction requires an object source")
                            .id as usize]
                            .zone
                            == Zone::Stack
                    {
                        self.move_object(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            Zone::Exile,
                        );
                    }
                }
                Instruction::PlayHideawayIfCoven => {
                    self.play_hideaway_if_coven(&item);
                    if self.state.resolution_free_casts.is_some() {
                        return;
                    }
                }
                Instruction::SelectLibraryThenMayTransform(effect) => {
                    if self.start_library_selection_completion(&item, effect, true) {
                        return;
                    }
                }
                Instruction::SelectLibraryTo(effect) => {
                    if self.start_library_selection_to(&item, effect) {
                        return;
                    }
                }
                Instruction::SelectLibrary(spec) => {
                    if self.start_library_selection(&item, spec) {
                        return;
                    }
                }
                Instruction::Scry(amount) | Instruction::Surveil(amount) => {
                    let remaining: Vec<_> = self.state.players[item.controller.index()]
                        .library
                        .iter()
                        .rev()
                        .filter(|&&id| self.state.objects[id as usize].is_card())
                        .take(usize::from(amount))
                        .copied()
                        .collect();
                    if !remaining.is_empty() {
                        self.state.resolving.as_mut().unwrap().library = Some(LibraryChoice {
                            surveil: matches!(instruction, Instruction::Surveil(_)),
                            return_within_life_gained: false,
                            return_noncreature_nonland: false,
                            remaining,
                            top: Vec::new(),
                            bottom: Vec::new(),
                            graveyard: Vec::new(),
                        });
                        self.state.pending = Pending::EffectLibrary {
                            player: item.controller,
                        };
                        return;
                    }
                    if amount != 0 {
                        self.scry_or_surveil_event(item.controller);
                    }
                }
                Instruction::SurveilOneReturnByLifeGained
                | Instruction::SurveilOneReturnNoncreatureNonland => {
                    let remaining: Vec<_> = self.state.players[item.controller.index()]
                        .library
                        .iter()
                        .rev()
                        .filter(|&&id| self.state.objects[id as usize].is_card())
                        .take(1)
                        .copied()
                        .collect();
                    if !remaining.is_empty() {
                        self.state.resolving.as_mut().unwrap().library = Some(LibraryChoice {
                            surveil: true,
                            return_within_life_gained: matches!(
                                instruction,
                                Instruction::SurveilOneReturnByLifeGained
                            ),
                            return_noncreature_nonland: matches!(
                                instruction,
                                Instruction::SurveilOneReturnNoncreatureNonland
                            ),
                            remaining,
                            top: Vec::new(),
                            bottom: Vec::new(),
                            graveyard: Vec::new(),
                        });
                        self.state.pending = Pending::EffectLibrary {
                            player: item.controller,
                        };
                        return;
                    }
                    self.scry_or_surveil_event(item.controller);
                }
                Instruction::CounterRelatedUnlessSacrifice(amount) => {
                    let related = item.related_stack.expect("ward payer and related object");
                    let mut payer_item = item.clone();
                    payer_item.controller = related.controller;
                    let pending = self.start_effect_choice(
                        &payer_item,
                        EffectChoiceKind::Sacrifice,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        amount,
                        Some(0),
                    );
                    if pending {
                        return;
                    }
                    self.counter_stack_item(related.id, false);
                }
                Instruction::CounterRelatedUnlessDiscard => {
                    if self.start_ward_discard_only(&item) {
                        return;
                    }
                }
                Instruction::CounterRelatedUnlessDiscardOrPay(cost) => {
                    if self.start_ward_discard_or_mana(&item, cost) {
                        return;
                    }
                }
                Instruction::CounterRelatedUnlessPayManaLife { cost, life } => {
                    if self.start_resolution_payment(&item, cost, CounterSubject::Related, false, 0)
                    {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .mana_payment
                            .as_mut()
                            .unwrap()
                            .life = Some(life);
                        return;
                    }
                }
                Instruction::PayXReflexive { .. } | Instruction::PayXReflexiveLimited { .. } => {
                    self.state.pending = Pending::ResolutionX {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::MayPayMana { cost, skip } => {
                    self.start_optional_resolution_payment(&item, cost, skip);
                    return;
                }
                Instruction::MayPayManaAlternatives { costs, skip } => {
                    let (&cost, alternatives) =
                        costs.split_first().expect("nonempty payment choices");
                    self.start_optional_resolution_payment(&item, cost, skip);
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .mana_payment
                        .as_mut()
                        .unwrap()
                        .alternatives = alternatives;
                    return;
                }
                Instruction::Endure { amount, token } => {
                    if amount != 0 {
                        if self.on_battlefield(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                        ) && self.object_ref(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                        ) == item
                            .source
                            .expect("instruction requires an object source")
                            .object
                        {
                            self.state.pending = Pending::Endure {
                                player: item.controller,
                                source: item
                                    .source
                                    .expect("instruction requires an object source")
                                    .object,
                                amount,
                                token,
                            };
                            return;
                        }
                        self.create_tokens(item.controller, token, 1, false);
                    }
                }
                Instruction::CreateTokensWithTemporaryKeywords {
                    token,
                    amount,
                    keywords,
                } => {
                    let amount = self.item_amount(amount, &item);
                    self.stage_token_creation(
                        item.controller,
                        token,
                        amount,
                        false,
                        replacements::TokenCompletion::Keywords {
                            source: item
                                .source
                                .expect("instruction requires an object source")
                                .object,
                            keywords,
                        },
                    );
                }
                Instruction::PutCombatDealerCounters => {
                    if let Some(event) = &item.creature_combat_event
                        && self.on_battlefield(event.dealer.card)
                        && self.object_ref(event.dealer.card) == event.dealer
                    {
                        let n = u64::try_from(event.amount).capacity("counter amount exhausted");
                        self.effect_counters(
                            vec![(event.dealer, Counters::of(CounterKind::PlusOne, n))],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::PutGrantorCounter(kind) => {
                    if let Some(grant) = &item.attachment_trigger
                        && self.on_battlefield(grant.grantor.card)
                        && self.object_ref(grant.grantor.card) == grant.grantor
                    {
                        self.effect_counters(
                            vec![(grant.grantor, Counters::of(kind, 1))],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::MayRemoveSourceCounter { kind, skip } => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.counter_count(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            kind,
                        ) > 0
                    {
                        let resolution = self.state.resolving.as_ref().unwrap();
                        let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                        assert!(skip_to <= resolution.program.len());
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    } else {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::RemoveSourceCounterIfAny { kind, skip } => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && self.counter_count(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            kind,
                        ) > 0
                    {
                        self.pay_counters(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            &Counters::of(kind, 1),
                        );
                    } else {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::CreateTokensForChosenCreatureType { .. }
                | Instruction::CreateEmblemForChosenCreatureType(_)
                | Instruction::DestroyCreaturesExceptChosenType => {
                    self.state.pending = Pending::CreatureTypeChoice {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::CreateTokensScaled {
                    token,
                    amount,
                    tapped,
                } => {
                    let amount = self.item_amount(amount, &item);
                    self.create_tokens(item.controller, token, amount, tapped);
                }
                Instruction::PutLandCards { maximum, tapped } => {
                    let maximum = u32::try_from(self.item_amount(maximum, &item))
                        .capacity("land selection limit exhausted");
                    if self.start_land_selection(&item, maximum, tapped) {
                        return;
                    }
                }
                Instruction::PutHandLand { tapped } => {
                    if self.start_hand_land(&item, tapped) {
                        return;
                    }
                }
                Instruction::PutHandLandWithLife {
                    tapped,
                    subtype,
                    life,
                } => {
                    if self.start_hand_land(&item, tapped) {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .choice
                            .as_mut()
                            .unwrap()
                            .entry_life_bonus = Some((subtype, life));
                        return;
                    }
                }
                Instruction::PutHandPermanent { maximum, tapped } => {
                    if self.start_hand_permanent(&item, maximum, tapped) {
                        return;
                    }
                }
                Instruction::PutHandAndAttachEquipment { filter } => {
                    if self.start_filtered_hand_entry(&item, filter, 0) {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .choice
                            .as_mut()
                            .unwrap()
                            .equipment_recipient = Some(
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                        );
                        return;
                    }
                }
                Instruction::PutHandFiltered { filter, skip } => {
                    if self.start_filtered_hand_entry(&item, filter, skip) {
                        return;
                    }
                }
                Instruction::PutHandCreature => {
                    if self.start_hand_creature(&item) {
                        return;
                    }
                }
                Instruction::DiscardManaValueLifeLoss => {
                    if self.start_discard_mana_value_life(&item) {
                        return;
                    }
                }
                Instruction::ExileIfDiesThisTurn => {
                    self.register_exile_if_dies(&item);
                }
                Instruction::CreateRoleReflexive { token, ability } => {
                    self.resolve_role_reflexive(&item, token, ability);
                }
                Instruction::EventObjectPowerDamage => self.event_object_power_damage(&item),
                Instruction::ReflexiveFight => {
                    self.reflexive_fight(&item);
                }
                Instruction::CreateRolesMatching { token, filter } => {
                    self.resolve_roles_matching(&item, token, filter);
                }
                Instruction::CreateRole { token, subject } => {
                    self.resolve_role(&item, token, subject);
                }
                Instruction::JobSelect { token } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        1,
                        false,
                        replacements::TokenCompletion::JobAttach(
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                        ),
                    );
                }
                Instruction::CreateTokensFor {
                    players,
                    token,
                    amount,
                    tapped,
                } => {
                    for player in self.program_players(players, &item).collect::<Vec<_>>() {
                        self.create_tokens(player, token, u128::from(amount), tapped);
                    }
                }
                Instruction::CreateEquipmentForSource { token, amount } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::AttachCreatedTo(
                            item.source
                                .expect("instruction requires an object source")
                                .object,
                        ),
                    );
                }
                Instruction::CreateTokenThenCounterOtherWizards { token } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        1,
                        false,
                        replacements::TokenCompletion::CounterOtherWizards,
                    );
                }
                Instruction::CreateTokensAttackThisCombat { token, amount } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::AttackThisCombat,
                    );
                }
                Instruction::CreateTokens {
                    token,
                    amount,
                    tapped,
                } => {
                    self.create_tokens(item.controller, token, u128::from(amount), tapped);
                }
                Instruction::ExileAndBorrowActivatedAbilities => {
                    self.exile_and_borrow_activated_abilities(&item)
                }
                Instruction::MemoryVessel => self.resolve_memory_vessel(&item),
                Instruction::RecordOncePerTurn => self.record_optional_ability_use(&item),
                Instruction::MayOncePerTurn { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if self.optional_ability_used(&item) {
                        self.state.resolving.as_mut().unwrap().pc = skip_to;
                    } else {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                }
                Instruction::May { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    self.state.pending = Pending::EffectOptional {
                        player: item.controller,
                        skip_to,
                    };
                    return;
                }
                Instruction::AirbendTargets => self.airbend_targets(&item),
                Instruction::AirbendOtherCreatures => {
                    let exception = item.targets.iter().find_map(|target| {
                        self.same_target_object(*target)
                            .then_some(target.object)
                            .flatten()
                    });
                    self.airbend_other_creatures(exception);
                }
                Instruction::ProhibitOpponentNonhandCastsUntilNextTurn => {
                    self.state
                        .roles
                        .get_or_insert_default()
                        .prohibit_opponent_nonhand_casts[item.controller.index()] = true;
                }
                Instruction::ExileTargetsForModifiedCast => {
                    self.exile_targets_for_modified_cast(&item)
                }
                Instruction::ExileTargetsPlay { next_turn } => {
                    self.exile_targets_play(&item, next_turn, false, false)
                }
                Instruction::ExileTargetsAndTopPlayForOwners => {
                    self.exile_targets_play(&item, true, true, true)
                }
                Instruction::ExileUntilSourceLeaves => self.exile_until_source_leaves(&item),
                Instruction::ExileBattlefieldUntilSourceLeaves { filters } => {
                    self.exile_battlefield_until_source_leaves(&item, filters)
                }
                Instruction::LinkedExileToGraveyardTokens { token } => {
                    self.linked_exile_to_graveyard_tokens(&item, token)
                }
                Instruction::ExileAndCaptureAttachedCopy => {
                    self.exile_and_capture_attached_copy(&item)
                }
                Instruction::ApplyCapturedAttachedCopy => self.apply_captured_attached_copy(),
                Instruction::ExileGraveyard(set) => {
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.zone == Zone::Graveyard
                                && object.is_card()
                                && players.contains(&object.owner)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::AddCastEntryCounters { kind, amount } => {
                    self.add_cast_entry_counters(&item, kind, amount)
                }
                Instruction::Recruit { token } => {
                    let player = item.controller;
                    self.draw_many(player, 1);
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards[player.index()] = 1;
                    resolution.discard_after =
                        Some(discard_events::DiscardAfter::Recruit(player, token));
                }
                Instruction::DrawThenDiscardIfDrew { amount, discard } => {
                    let player = item.controller;
                    let before = self.state.players[player.index()].cards_drawn_this_turn;
                    self.draw_many(player, self.item_amount(amount, &item));
                    if self.state.players[player.index()].cards_drawn_this_turn > before {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] =
                            usize::from(discard);
                    }
                }
                Instruction::DrawPowerOfTwoChosenX(set) => {
                    let amount = 1u128.checked_shl(item.x.unwrap_or(0)).unwrap_or(u128::MAX);
                    for player in self.program_players(set, &item) {
                        self.draw_many(player, amount);
                    }
                }
                Instruction::Draw(set, _) | Instruction::DrawScaled(set, _) => {
                    let amount = match instruction {
                        Instruction::Draw(_, amount) => u128::from(amount),
                        Instruction::DrawScaled(_, value) => self.item_amount(value, &item),
                        _ => unreachable!(),
                    };
                    for player in self.program_players(set, &item) {
                        self.draw_many(player, amount);
                    }
                }
                Instruction::DrawGreatestCreaturePowerRemember => {
                    let player = item.controller;
                    self.state.resolving.as_mut().unwrap().remembered_draws = Some(0);
                    let greatest = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == player
                                && self.is_creature(*id as CardId)
                        })
                        .map(|(id, _)| self.stats(id as CardId).power)
                        .max()
                        .unwrap_or(0)
                        .max(0);
                    self.draw_many_recording(
                        player,
                        u128::try_from(greatest).expect("nonnegative power"),
                        item.id,
                    );
                }
                Instruction::LoseLifeRememberedDraws => {
                    let player = item.controller;
                    let count = self
                        .state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .remembered_draws
                        .take()
                        .expect("remembered draw must precede its continuation");
                    self.lose_life(player, i128::from(count));
                }
                Instruction::LoseOpponentGainSame(value) => {
                    let amount = i128::try_from(self.item_amount(value, &item))
                        .capacity("life quantity representation exhausted");
                    self.lose_life(item.controller.other(), amount);
                    self.gain_life(item.controller, amount);
                }
                Instruction::DrainLifeScaled(value) => {
                    let amount = i128::try_from(self.item_amount(value, &item))
                        .capacity("life drain amount representation exhausted");
                    let opponent = item.controller.other();
                    let before = self.state.players[opponent.index()].life_lost_this_turn;
                    self.lose_life(opponent, amount);
                    let lost = self.state.players[opponent.index()].life_lost_this_turn - before;
                    self.gain_life(
                        item.controller,
                        i128::try_from(lost).capacity("life drain result exhausted"),
                    );
                }
                Instruction::DiscardRandom(set, amount) => {
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    let mut selected = Vec::new();
                    for player in players {
                        let mut hand: Vec<_> = self
                            .state
                            .objects
                            .iter()
                            .enumerate()
                            .filter_map(|(id, o)| {
                                (o.zone == Zone::Hand && o.owner == player && o.is_card())
                                    .then_some(id as CardId)
                            })
                            .collect();
                        for _ in 0..usize::from(amount).min(hand.len()) {
                            let index = if hand.len() == 1 {
                                0
                            } else {
                                self.state.rng.below(hand.len() as u64) as usize
                            };
                            selected.push(hand.swap_remove(index));
                        }
                    }
                    self.discard_batch_caused(&selected, None, Some(item.controller));
                }
                Instruction::DiscardTwoNonlandReflexive(ability) => {
                    self.state.resolving.as_mut().unwrap().discards[item.controller.index()] = 2;
                    self.state.resolving.as_mut().unwrap().discard_after =
                        Some(discard_events::DiscardAfter::NonlandReflexive {
                            controller: item.controller,
                            ability,
                        });
                }
                Instruction::RevealTwoOutsideCards => {
                    if self.start_outside_choice(item.controller) {
                        return;
                    }
                }
                Instruction::RevealTwoOutsideCardsWithSelf(effects) => {
                    if self.start_outside_choice_with_self(item.controller, item.id, effects) {
                        return;
                    }
                }
                Instruction::OpponentsDiscardTwoDrawUnlessTwoNonland => {
                    self.state.resolving.as_mut().unwrap().discards
                        [item.controller.other().index()] = 2;
                    self.state.resolving.as_mut().unwrap().discard_after = Some(
                        discard_events::DiscardAfter::DrawUnlessOpponentDiscardedTwoNonland(
                            item.controller,
                        ),
                    );
                }
                Instruction::Discard(set, _) | Instruction::DiscardScaled(set, _) => {
                    let amount = match instruction {
                        Instruction::Discard(_, n) => usize::from(n),
                        Instruction::DiscardScaled(_, value) => {
                            usize::try_from(self.item_amount(value, &item))
                                .capacity("discard amount exhausted")
                        }
                        _ => unreachable!(),
                    };
                    for player in self.program_players(set, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] = amount;
                    }
                }
                Instruction::CreateEmblem(factory) => self.create_emblem(item.controller, factory),
                Instruction::GrantNextSpellUncounterable => {
                    self.grant_next_spell_uncounterable(item.controller)
                }
                Instruction::EndTurn => {
                    self.end_turn_effect();
                    return;
                }
                Instruction::WinIfLibraryEmpty => {
                    if self.library_card_count(item.controller) == 0 {
                        self.try_win_game(item.controller);
                        if self.state.outcome.is_some() {
                            self.state.resolving = None;
                            return;
                        }
                    }
                }
                Instruction::WinIfDistinctSubtypeNames { subtype, minimum } => {
                    if self.distinct_subtype_names(item.controller, subtype) >= usize::from(minimum)
                    {
                        self.try_win_game(item.controller);
                        if self.state.outcome.is_some() {
                            self.state.resolving = None;
                            return;
                        }
                    }
                }
                Instruction::LoseGame(set) => {
                    let mut lost = [false; 2];
                    for player in self.program_players(set, &item) {
                        lost[player.index()] = true;
                    }
                    self.commit_game_losses(self.allowed_game_losses(lost));
                    if self.state.outcome.is_some() {
                        self.state.resolving = None;
                        self.state.pending = Pending::Terminal;
                        return;
                    }
                }
                Instruction::LoseGainedLife(set) => {
                    let amount = item
                        .life_event
                        .expect("life-event program requires context")
                        .amount;
                    for player in self.program_players(set, &item) {
                        self.lose_life(player, amount);
                    }
                }
                Instruction::GainLife(set, _) | Instruction::GainLifeScaled(set, _) => {
                    let amount = match instruction {
                        Instruction::GainLife(_, amount) => i128::from(amount),
                        Instruction::GainLifeScaled(_, value) => {
                            i128::try_from(self.item_amount(value, &item))
                                .capacity("life gain amount representation exhausted")
                        }
                        _ => unreachable!(),
                    };
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    for player in players {
                        self.gain_life(player, amount);
                    }
                }
                Instruction::LoseLifeTargetController(amount)
                | Instruction::DamageTargetController(amount) => {
                    let resolution = self.state.resolving.as_ref().unwrap();
                    let players: Vec<_> =
                        resolution
                            .target_card_facts
                            .as_ref()
                            .map_or_else(Vec::new, |facts| {
                                facts
                                    .iter()
                                    .filter(|facts| facts.group == resolution.target_group)
                                    .map(|facts| facts.controller)
                                    .collect()
                            });
                    if matches!(instruction, Instruction::LoseLifeTargetController(_)) {
                        for player in players {
                            self.lose_life(player, i128::from(amount));
                        }
                    } else {
                        let source = self.program_damage_source(&item);
                        let events: Vec<_> = players
                            .into_iter()
                            .map(|player| DamageEvent {
                                source,
                                target: Target::Player(player),
                                amount: i128::from(amount),
                                combat: false,
                                preventable: true,
                            })
                            .collect();
                        self.apply_damage_batch(&events);
                    }
                }
                Instruction::LoseLifeTargetCardManaValue(set) => {
                    let resolution = self.state.resolving.as_ref().unwrap();
                    let amount = resolution.target_card_facts.as_ref().map_or(0, |values| {
                        values
                            .iter()
                            .filter(|facts| facts.group == resolution.target_group)
                            .map(|facts| i128::from(facts.mana_value))
                            .sum()
                    });
                    for player in self.program_players(set, &item) {
                        self.lose_life(player, amount);
                    }
                }
                Instruction::LoseLifeScaled(set, value) => {
                    let amount = i128::try_from(self.item_amount(value, &item))
                        .capacity("life loss amount exhausted");
                    for player in self.program_players(set, &item) {
                        self.lose_life(player, amount);
                    }
                }
                Instruction::CombatDamageByGreaterToughnessThisTurn => {
                    self.state
                        .roles
                        .get_or_insert_default()
                        .toughness_combat_damage[item.controller.index()] = true;
                }
                Instruction::DrainOpponents(amount) => {
                    let other = item.controller.other();
                    let before = self.state.players[other.index()].life;
                    self.lose_life(other, i128::from(amount));
                    let lost = before
                        .checked_sub(self.state.players[other.index()].life)
                        .capacity("life drain representation exhausted")
                        .max(0);
                    if lost > 0 {
                        self.gain_life(item.controller, lost);
                    }
                }
                Instruction::LoseLife(set, amount) => {
                    // These paired instructions represent an accepted payment,
                    // while ordinary LoseLife remains an unreplaced loss.
                    let life_payment = resolution.pc >= 2
                        && matches!(
                            resolution.program[resolution.pc - 2],
                            Instruction::MayPayOpponentLife { .. }
                                | Instruction::MayPayControllerLife { .. }
                        );
                    for player in self.program_players(set, &item) {
                        if life_payment {
                            self.pay_life(player, i128::from(amount));
                        } else {
                            self.lose_life(player, i128::from(amount));
                        }
                    }
                }
                Instruction::UntapPlayerCreatures(players) => {
                    let recipients = self.player_creature_recipients(players, &item);
                    self.untap_objects(&recipients, untap_rules::UntapCause::Effect);
                }
                Instruction::PutOtherControlledCreaturesExceptSource { kind, amount } => {
                    let changes = self
                        .player_creature_recipients(PlayerSet::Controller, &item)
                        .into_iter()
                        .filter(|&id| {
                            self.object_ref(id)
                                != item
                                    .source
                                    .expect("instruction requires an object source")
                                    .object
                        })
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutOtherControlledCreatureCounters { kind, amount } => {
                    let changes = self
                        .player_creature_recipients(PlayerSet::Controller, &item)
                        .into_iter()
                        .filter(|&id| {
                            !item
                                .targets
                                .iter()
                                .any(|t| t.object == Some(self.object_ref(id)))
                        })
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutPlayerCreatureCounters {
                    players,
                    kind,
                    amount,
                } => {
                    let changes = self
                        .player_creature_recipients(players, &item)
                        .into_iter()
                        .map(|id| {
                            let mut counters = Counters::default();
                            counters.add(kind, u64::from(amount));
                            (self.object_ref(id), counters)
                        })
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutControlledCreatureCountersWith {
                    required,
                    kind,
                    amount,
                } => {
                    let changes = self
                        .player_creature_recipients(PlayerSet::Controller, &item)
                        .into_iter()
                        .filter(|&id| self.counter_count(id, required) > 0)
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::GrantLandManaUntilSourceCast => {
                    self.grant_land_mana_until_paid_source_cast(&item)
                }
                Instruction::UntapPermanentGroup(filters) => {
                    let recipients = self.battlefield_group(&item, filters);
                    self.untap_objects(&recipients, untap_rules::UntapCause::Effect);
                }
                Instruction::GrantCounterBearingCreatures(keywords) => {
                    let recipients: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, object)| {
                            let id = id as CardId;
                            (object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.is_creature(id)
                                && self.counter_amounts(id).has_any())
                            .then_some(id)
                        })
                        .collect();
                    if let Some(source) = item.source {
                        self.grant_keyword_effects(&recipients, source.object, keywords);
                    }
                }
                Instruction::RemovePermanentGroupKeywords { filters, keywords } => {
                    self.remove_permanent_group_keywords(&item, filters, keywords);
                }
                Instruction::UntapControlledLands => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, object)| {
                            (object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.current_kinds(id as CardId) & 2 != 0)
                                .then_some(id as CardId)
                        })
                        .collect();
                    self.untap_objects(&ids, untap_rules::UntapCause::Effect);
                }
                Instruction::UntapControlledTokens => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(id, o)| {
                            (o.zone == Zone::Battlefield
                                && o.controller == item.controller
                                && o.is_token())
                            .then_some(id as CardId)
                        })
                        .collect();
                    self.untap_objects(&ids, untap_rules::UntapCause::Effect);
                }
                Instruction::UntapCreatures(filter) | Instruction::TapCreatures(filter) => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, _)| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                *id as CardId,
                            )
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    if matches!(instruction, Instruction::TapCreatures(_)) {
                        self.tap_objects(&ids);
                    } else {
                        self.untap_objects(&ids, untap_rules::UntapCause::Effect);
                    }
                }
                Instruction::UntapEventObject => {
                    // An untargeted event object remains identified even if it stops
                    // being a creature, or the Equipment leaves or attaches elsewhere.
                    let recipient = item.entering_object.or_else(|| {
                        item.attachment_trigger
                            .as_ref()
                            .map(|event| event.recipient)
                    });
                    if let Some(id) = recipient
                        .filter(|object| {
                            self.on_battlefield(object.card)
                                && self.object_ref(object.card) == *object
                        })
                        .map(|object| object.card)
                    {
                        self.untap_objects(&[id], untap_rules::UntapCause::Effect);
                    }
                }
                Instruction::UntapSource | Instruction::TapSource => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        if instruction == Instruction::TapSource {
                            self.tap_objects(&[item
                                .source
                                .expect("instruction requires an object source")
                                .id]);
                        } else {
                            self.untap_objects(
                                &[item
                                    .source
                                    .expect("instruction requires an object source")
                                    .id],
                                untap_rules::UntapCause::Effect,
                            );
                        }
                    }
                }
                Instruction::DamageEachPlayerPerNonbasicLand => {
                    let source = self.program_damage_source(&item);
                    let events: Vec<_> = [Player::P0, Player::P1]
                        .into_iter()
                        .map(|player| {
                            let amount = self
                                .state
                                .objects
                                .iter()
                                .enumerate()
                                .filter(|(id, object)| {
                                    object.zone == Zone::Battlefield
                                        && object.controller == player
                                        && self.current_kinds(*id as CardId) & 2 != 0
                                        && !object.card.is_basic_land()
                                })
                                .count();
                            DamageEvent {
                                preventable: true,
                                source,
                                target: Target::Player(player),
                                amount: amount as i128,
                                combat: false,
                            }
                        })
                        .collect();
                    self.apply_damage_batch(&events);
                }
                Instruction::Damage(_)
                | Instruction::DamageScaled(_)
                | Instruction::DamagePlayers(_, _)
                | Instruction::DamagePlayersScaled(_, _)
                | Instruction::DamageDiscardAmount(_) => {
                    let amount = match instruction {
                        Instruction::Damage(amount) | Instruction::DamagePlayers(_, amount) => {
                            i128::from(amount)
                        }
                        Instruction::DamageScaled(value)
                        | Instruction::DamagePlayersScaled(_, value) => {
                            i128::try_from(self.item_amount(value, &item))
                                .capacity("damage amount representation exhausted")
                        }
                        Instruction::DamageDiscardAmount(_) => i128::from(
                            item.discard_event
                                .expect("discard-sized effect requires event")
                                .amount,
                        ),
                        _ => unreachable!(),
                    };
                    let source = self.program_damage_source(&item);
                    let event = |target| DamageEvent {
                        preventable: true,
                        source,
                        target,
                        amount,
                        combat: false,
                    };
                    let events: Vec<_> = if let Instruction::DamagePlayers(players, _)
                    | Instruction::DamagePlayersScaled(players, _)
                    | Instruction::DamageDiscardAmount(players) =
                        instruction
                    {
                        self.program_players(players, &item)
                            .map(|player| event(Target::Player(player)))
                            .collect()
                    } else {
                        item.targets
                            .iter()
                            .copied()
                            .filter(|&bound| self.same_target_object(bound))
                            .map(|bound| event(bound.target))
                            .collect()
                    };
                    self.apply_damage_batch(&events);
                }
                Instruction::SetBaseStats {
                    subject,
                    power,
                    toughness,
                } => self.set_base_stats(&item, subject, power, toughness),
                Instruction::SetBaseStatsFromSource { subject } => {
                    self.set_base_stats_from_source(&item, subject);
                }
                Instruction::SetPermanentGroupBaseStats {
                    filters,
                    exclude_source,
                    power,
                    toughness,
                } => {
                    self.set_permanent_group_base_stats(
                        &item,
                        filters,
                        exclude_source,
                        power,
                        toughness,
                    );
                }
                Instruction::SetResolvedBaseStats {
                    subject,
                    power,
                    toughness,
                } => {
                    self.set_resolved_base_stats(&item, subject, power, toughness);
                }
                Instruction::Grow {
                    subject,
                    power,
                    toughness,
                }
                | Instruction::GrowScaled {
                    subject,
                    power,
                    toughness,
                    ..
                }
                | Instruction::GrowResolved {
                    subject,
                    power,
                    toughness,
                    ..
                } => {
                    let amount = match instruction {
                        Instruction::GrowScaled { amount, .. } => {
                            i128::try_from(self.item_amount(amount, &item))
                                .capacity("growth amount representation exhausted")
                        }
                        Instruction::GrowResolved { value, .. } => {
                            self.resolved_stat_value(&item, value).unwrap_or(0).max(0)
                        }
                        _ => 1,
                    };
                    let stats = Stats {
                        power: i128::from(power)
                            .checked_mul(amount)
                            .capacity("power representation exhausted"),
                        toughness: i128::from(toughness)
                            .checked_mul(amount)
                            .capacity("toughness representation exhausted"),
                    };
                    match subject {
                        Subject::Source => {
                            if self.on_battlefield(
                                item.source
                                    .expect("instruction requires an object source")
                                    .id,
                            ) && self.object_ref(
                                item.source
                                    .expect("instruction requires an object source")
                                    .id,
                            ) == item
                                .source
                                .expect("instruction requires an object source")
                                .object
                            {
                                self.state.objects[item
                                    .source
                                    .expect("instruction requires an object source")
                                    .id
                                    as usize]
                                    .growth += stats;
                            }
                        }
                        Subject::Attached => {
                            if let Some(id) = self.attached_subject(&item) {
                                self.state.objects[id as usize].growth += stats;
                            }
                        }
                        Subject::Entering => {
                            if let Some(id) = self.entering_subject(&item) {
                                self.state.objects[id as usize].growth += stats;
                            }
                        }
                        Subject::Targets => {
                            for bound in &item.targets {
                                if self.same_target_object(*bound)
                                    && let Target::Card(id) = bound.target
                                {
                                    self.state.objects[id as usize].growth += stats;
                                }
                            }
                        }
                    }
                }
                Instruction::EvolveSource {
                    required,
                    types,
                    stats,
                    keywords,
                    ability,
                } => {
                    self.evolve_source(&item, required, types, stats, keywords, ability);
                }
                Instruction::AddCreatureTypes { subject, types } => {
                    self.add_creature_types(&item, subject, types)
                }
                Instruction::GrantAllCreatureTypes(subject) => {
                    self.grant_all_creature_types(&item, subject)
                }
                Instruction::AddBasicLandType { subject, kind } => {
                    self.add_basic_land_type(&item, subject, kind);
                }
                Instruction::SetCreatureTypes { subject, types } => {
                    self.set_creature_types(&item, subject, types);
                }
                Instruction::SacrificeAbilitySource => {
                    let source = item.target_source();
                    if self.on_battlefield(source.card)
                        && self.object_ref(source.card) == source
                        && self.state.objects[source.card as usize].controller == item.controller
                    {
                        self.sacrifice_batch(&[(item.controller, source.card)]);
                    }
                }
                Instruction::ExileSourceGainLifeIfMoved(life) => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        let event = self.move_object(
                            item.source
                                .expect("instruction requires an object source")
                                .object
                                .card,
                            Zone::Exile,
                        );
                        if event.moved && event.destination == Zone::Exile {
                            self.gain_life(item.controller, i128::from(life));
                        }
                    }
                }
                Instruction::GrantCreatureAbility { filter, ability } => {
                    let ids = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.on_battlefield(id)
                                && self.is_creature(id)
                                && self.creature_matches(
                                    filter,
                                    item.controller,
                                    item.source
                                        .expect("instruction requires an object source")
                                        .object,
                                    id,
                                )
                        })
                        .collect::<Vec<_>>();
                    self.grant_temporary_ability_to(&item, ability, &ids);
                }
                Instruction::GrantUntilEndTurn { ability } => {
                    self.grant_temporary_ability(&item, ability)
                }
                Instruction::ReturnGrantedSource { counter, role } => {
                    self.return_granted_source(&item, counter, role)
                }
                Instruction::GrantSelfAbility(kind) => self.grant_self_ability(&item, kind),
                Instruction::GrantSourceTriggerUntilEndOfTurn(ability) => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        self.grant_temporary_ability_to(
                            &item,
                            ability,
                            &[item
                                .source
                                .expect("instruction requires an object source")
                                .object
                                .card],
                        );
                    }
                }
                Instruction::GrantSourcePermanentTrigger(ability) => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        let timestamp = self.next_role_timestamp();
                        self.grant_source_permanent_trigger(&item, ability, timestamp);
                    }
                }
                Instruction::GrantPermanentSelfAbility(kind) => {
                    self.grant_permanent_self_ability(&item, kind)
                }
                Instruction::GrantCastCreatureKeywords(keywords) => {
                    self.grant_cast_creature_keywords(&item, keywords);
                }
                Instruction::RegisterEarthbendReturn => self.register_earthbend_return(&item),
                Instruction::AnimateEarthbendTargets => self.animate_earthbend_targets(&item),
                Instruction::GrantWarpedTargetsKeywords(keywords) => {
                    let ids: Vec<_> = self
                        .keyword_subjects(&item, Subject::Targets)
                        .into_iter()
                        .filter(|&id| self.state.objects[id as usize].cast_for_warp)
                        .collect();
                    self.grant_keyword_effects(
                        &ids,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        keywords,
                    );
                }
                Instruction::GrantKeywords { subject, keywords } => {
                    let ids = self.keyword_subjects(&item, subject);
                    self.grant_keyword_effects(
                        &ids,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        keywords,
                    );
                }
                Instruction::GrantPermanentGroupKeywords { filters, keywords } => {
                    self.grant_permanent_group_keywords(&item, filters, keywords);
                }
                Instruction::GrantCreatures { filter, keywords } => {
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                id,
                            )
                        })
                        .collect();
                    let source = item
                        .source
                        .expect("instruction requires an object source")
                        .object;
                    self.grant_keyword_effects(&affected, source, keywords);
                }
                Instruction::GrantRememberedCreaturesUntilNextTurn(keywords) => {
                    let objects = std::mem::take(
                        &mut self.state.resolving.as_mut().unwrap().remembered_creatures,
                    );
                    let affected: Vec<_> = objects
                        .into_iter()
                        .filter(|object| {
                            self.on_battlefield(object.card)
                                && self.object_ref(object.card) == *object
                        })
                        .map(|object| object.card)
                        .collect();
                    let source = item
                        .source
                        .expect("instruction requires an object source")
                        .object;
                    self.grant_keywords_until_next_turn(
                        &affected,
                        source,
                        item.controller,
                        keywords,
                    );
                }
                Instruction::SnapshotDoubleCreatures(filter) => {
                    // Snapshot every operand before changing any derived characteristic.
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                id,
                            )
                        })
                        .map(|id| (id, self.stats(id)))
                        .collect();
                    for (id, delta) in affected {
                        self.state.objects[id as usize].growth += delta;
                    }
                }
                Instruction::GrowPermanentGroup {
                    filters,
                    power,
                    toughness,
                } => {
                    self.grow_permanent_group(&item, filters, power, toughness);
                }
                Instruction::GrowPermanentGroupScaled {
                    filters,
                    exclude_source,
                    amount,
                } => {
                    self.grow_permanent_group_scaled(&item, filters, exclude_source, amount);
                }
                Instruction::GrowCreaturesScaled {
                    filter,
                    excluding_subtype,
                    amount,
                    power,
                    toughness,
                } => {
                    let amount = i128::try_from(self.item_amount(amount, &item))
                        .capacity("growth amount representation exhausted");
                    let delta = Stats {
                        power: i128::from(power)
                            .checked_mul(amount)
                            .capacity("power representation exhausted"),
                        toughness: i128::from(toughness)
                            .checked_mul(amount)
                            .capacity("toughness representation exhausted"),
                    };
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                id,
                            ) && excluding_subtype
                                .is_none_or(|subtype| !self.has_subtype(id, subtype))
                        })
                        .collect();
                    for id in affected {
                        self.state.objects[id as usize].growth += delta;
                    }
                }
                Instruction::GrowPlayerCreatures {
                    players,
                    power,
                    toughness,
                } => {
                    let recipients = self.player_creature_recipients(players, &item);
                    for id in recipients {
                        self.state.objects[id as usize].growth += Stats {
                            power: i128::from(power),
                            toughness: i128::from(toughness),
                        };
                    }
                }
                Instruction::GrowCreatures {
                    filter,
                    power,
                    toughness,
                } => {
                    // Lock the affected set at resolution; future entrants get no bonus.
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                id,
                            )
                        })
                        .collect();
                    for id in affected {
                        self.state.objects[id as usize].growth += Stats {
                            power: i128::from(power),
                            toughness: i128::from(toughness),
                        };
                    }
                }
                Instruction::PutDiscardCounters(kind) => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .id,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                    {
                        self.effect_counter(
                            item.source
                                .expect("instruction requires an object source")
                                .id,
                            kind,
                            item.discard_event.expect("discard event").amount.into(),
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::PutFilteredCounters {
                    filter,
                    kind,
                    amount,
                } => {
                    let affected = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            Self::count_object_matches(
                                filter,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                item.controller,
                                *id as CardId,
                                object,
                                &|id| self.granted_subtypes(id),
                                &|id, object| self.object_kinds(id, object),
                            )
                        })
                        .map(|(id, _)| {
                            (
                                self.object_ref(id as CardId),
                                Counters::of(kind, u64::from(amount)),
                            )
                        })
                        .collect();
                    self.effect_counters(affected, replacements::ResumeMode::Program);
                }
                Instruction::PutCreatureCounters {
                    filter,
                    kind,
                    amount,
                }
                | Instruction::PutCreatureCountersAndRemember {
                    filter,
                    kind,
                    amount,
                } => {
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(
                                filter,
                                item.controller,
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                id,
                            )
                        })
                        .map(|id| {
                            let mut c = Counters::default();
                            c.add(kind, u64::from(amount));
                            (self.object_ref(id), c)
                        })
                        .collect();
                    if matches!(
                        instruction,
                        Instruction::PutCreatureCountersAndRemember { .. }
                    ) {
                        self.state.resolving.as_mut().unwrap().remembered_creatures =
                            affected.iter().map(|(object, _)| *object).collect();
                    }
                    self.effect_counters(affected, replacements::ResumeMode::Program);
                }
                Instruction::PutPowerExcessCounters { filter } => {
                    self.put_power_excess_counters(&item, filter);
                }
                Instruction::DoubleCounterKind { subject, kind } => {
                    let changes = self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .map(|id| {
                            let mut counters = Counters::default();
                            counters.add(kind, self.counter_count(id, kind));
                            (self.object_ref(id), counters)
                        })
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::ReturnLinkedGhosts => {
                    let ids = self.linked_exile_candidates(
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                    );
                    let moves = ids
                        .into_iter()
                        .filter(|r| self.current_kinds(r.card) & 1 != 0)
                        .map(|r| {
                            let mut movement = self.entry_move(r.card, Zone::Battlefield, false);
                            movement.controller = item.controller;
                            movement.effect_counter = Some((CounterKind::Flying, 1));
                            movement.ghost_vacuum_after = Some(
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                            );
                            movement
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ExileGraveTargetsLinked => {
                    let ids: Vec<_> = item
                        .targets
                        .iter()
                        .filter(|b| self.same_target_object(**b))
                        .filter_map(|b| match b.target {
                            Target::Card(id)
                                if self.state.objects[id as usize].zone == Zone::Graveyard =>
                            {
                                Some(id)
                            }
                            _ => None,
                        })
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    for event in &events {
                        if event.moved && event.destination == Zone::Exile {
                            self.state
                                .linked_choices
                                .get_or_insert_default()
                                .links
                                .push(linked_choices::LinkedExileView {
                                    source: item
                                        .source
                                        .expect("instruction requires an object source")
                                        .object,
                                    exiled: event.after,
                                });
                        }
                    }
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::ExileGraveTargets => {
                    let ids: Vec<_> = item
                        .targets
                        .iter()
                        .filter(|b| self.same_target_object(**b))
                        .filter_map(|b| match b.target {
                            Target::Card(id)
                                if self.state.objects[id as usize].zone == Zone::Graveyard =>
                            {
                                Some(id)
                            }
                            _ => None,
                        })
                        .collect();
                    let creatures: Vec<_> = ids
                        .iter()
                        .copied()
                        .filter(|&id| {
                            self.state.objects[id as usize].card.is_creature()
                                && !self.state.objects[id as usize].is_token()
                        })
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    let count = events
                        .iter()
                        .filter(|e| {
                            e.moved
                                && e.destination == Zone::Exile
                                && creatures.contains(&e.before.id)
                        })
                        .count();
                    self.state.resolving.as_mut().unwrap().moved_count = Some(count as u64);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::IfExiledCreature { skip } => {
                    if self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .moved_count
                        .unwrap_or(0)
                        == 0
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::GainMissingTargetKeywordCounters => {
                    if self.on_battlefield(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) && self.object_ref(
                        item.source
                            .expect("instruction requires an object source")
                            .object
                            .card,
                    ) == item
                        .source
                        .expect("instruction requires an object source")
                        .object
                        && let Some(target) = item.targets.iter().find_map(|bound| {
                            if self.same_target_object(*bound)
                                && let Target::Card(id) = bound.target
                                && self.on_battlefield(id)
                                && self.is_creature(id)
                            {
                                Some(id)
                            } else {
                                None
                            }
                        })
                    {
                        let donor = self.keywords(target);
                        let existing = self.keywords(
                            item.source
                                .expect("instruction requires an object source")
                                .object
                                .card,
                        );
                        let mut counters = Counters::default();
                        for kind in CounterKind::KEYWORDS {
                            let keyword = kind.keyword().unwrap();
                            if donor.contains(keyword) && !existing.contains(keyword) {
                                counters = counters.with(kind, 1);
                            }
                        }
                        self.effect_counters(
                            vec![(
                                item.source
                                    .expect("instruction requires an object source")
                                    .object,
                                counters,
                            )],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::AddOneOfEachCounterKind => {
                    let mut changes = Vec::new();
                    let mut player_counters = [[0; 5]; 2];
                    for bound in &item.targets {
                        if !self.same_target_object(*bound) {
                            continue;
                        }
                        match bound.target {
                            Target::Player(player) => {
                                for kind in crate::cards::PlayerCounterKind::ALL {
                                    if self.state.players[player.index()].player_counter(kind) > 0 {
                                        player_counters[player.index()][kind.index()] = 1;
                                    }
                                }
                            }
                            Target::Card(id) if self.on_battlefield(id) => {
                                let mut add = Counters::default();
                                for kind in CounterKind::ALL {
                                    if self.counter_count(id, kind) > 0 {
                                        add = add.with(kind, 1);
                                    }
                                }
                                changes.push((self.object_ref(id), add));
                            }
                            _ => {}
                        }
                    }
                    self.effect_counters_with_players(
                        changes,
                        player_counters,
                        item.controller,
                        replacements::ResumeMode::Program,
                    );
                }
                Instruction::DoubleCounters => {
                    // Doubling puts the existing quantity of each kind on the object.
                    // Replacement effects modify that added quantity, not the old total.
                    let changes = item
                        .targets
                        .iter()
                        .filter_map(|b| {
                            if !self.same_target_object(*b) {
                                return None;
                            }
                            let Target::Card(id) = b.target else {
                                return None;
                            };
                            Some((self.object_ref(id), self.counter_amounts(id).to_owned()))
                        })
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::RemoveAllCountersFrom(subject) => {
                    let ids = if subject == Subject::Attached {
                        self.attached_permanent(&item).into_iter().collect()
                    } else {
                        self.keyword_subjects(&item, subject)
                    };
                    for id in ids {
                        self.set_counters(id, Counters::default());
                    }
                }
                Instruction::Proliferate(amount) => {
                    let count = self.amount_value(
                        amount,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        item.controller,
                    );
                    self.start_proliferate(item.controller, count);
                }
                Instruction::RemoveAllCounters => {
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            self.set_counters(id, Counters::default());
                        }
                    }
                }
                Instruction::PutCountersOnTappedControlledCreatures { kind, amount } => {
                    let changes = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            let object = &self.state.objects[id as usize];
                            object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && object.tapped
                                && self.current_kinds(id) & 1 != 0
                        })
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutCountersOnTargetGroup {
                    group,
                    kind,
                    amount,
                } => {
                    let changes = item
                        .targets
                        .iter()
                        .filter_map(|bound| {
                            if bound.group != group || !self.same_target_object(*bound) {
                                return None;
                            }
                            let Target::Card(id) = bound.target else {
                                return None;
                            };
                            self.on_battlefield(id).then(|| {
                                (self.object_ref(id), Counters::of(kind, u64::from(amount)))
                            })
                        })
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutCountersOnTargetKinds {
                    kinds,
                    kind,
                    amount,
                }
                | Instruction::PutCountersOnTargetAllKinds {
                    kinds,
                    kind,
                    amount,
                } => {
                    let changes = item
                        .targets
                        .iter()
                        .filter_map(|bound| {
                            let Target::Card(id) = bound.target else {
                                return None;
                            };
                            (self.same_target_object(*bound)
                                && self.on_battlefield(id)
                                && if matches!(
                                    instruction,
                                    Instruction::PutCountersOnTargetAllKinds { .. }
                                ) {
                                    self.current_kinds(id) & kinds == kinds
                                } else {
                                    self.current_kinds(id) & kinds != 0
                                })
                            .then(|| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        })
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutCounters { .. }
                | Instruction::PutCountersScaled { .. }
                | Instruction::PutCountersMixed { .. } => {
                    let (subject, counters) = match instruction {
                        Instruction::PutCounters {
                            subject,
                            kind,
                            amount,
                        } => (subject, Counters::of(kind, u64::from(amount))),
                        Instruction::PutCountersScaled {
                            subject,
                            kind,
                            amount,
                        } => (
                            subject,
                            Counters::of(
                                kind,
                                u64::try_from(self.item_amount(amount, &item))
                                    .capacity("counter amount exhausted"),
                            ),
                        ),
                        Instruction::PutCountersMixed { subject, amounts } => {
                            let mut counters = Counters::default();
                            for &(kind, amount) in amounts {
                                counters.add(kind, u64::from(amount));
                            }
                            (subject, counters)
                        }
                        _ => unreachable!(),
                    };
                    let ids: Vec<_> = match subject {
                        Subject::Entering => self.entering_subject(&item).into_iter().collect(),
                        Subject::Attached => self.attached_subject(&item).into_iter().collect(),
                        Subject::Source => {
                            if self.on_battlefield(
                                item.source
                                    .expect("instruction requires an object source")
                                    .id,
                            ) && self.object_ref(
                                item.source
                                    .expect("instruction requires an object source")
                                    .id,
                            ) == item
                                .source
                                .expect("instruction requires an object source")
                                .object
                            {
                                vec![
                                    item.source
                                        .expect("instruction requires an object source")
                                        .id,
                                ]
                            } else {
                                Vec::new()
                            }
                        }
                        Subject::Targets => item
                            .targets
                            .iter()
                            .filter_map(|b| {
                                if self.same_target_object(*b) {
                                    if let Target::Card(id) = b.target {
                                        Some(id)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    };
                    let changes = ids
                        .into_iter()
                        .map(|id| (self.object_ref(id), counters.clone()))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::GainControlAndPreventCombatWhileSourceControlled => {
                    self.resolve_source_linked_effect(
                        &item,
                        SourceLinkedEffectKind::ControlAndCombatLockWhileControlled,
                    );
                }
                Instruction::GainControlTargetsUntilSourceLeaves => {
                    self.resolve_source_linked_effect(&item, SourceLinkedEffectKind::Control);
                }
                Instruction::RemoveTargetAbilitiesUntilEndOfTurn => {
                    self.remove_target_abilities_until_end_of_turn(&item);
                }
                Instruction::RemoveTargetAbilitiesUntilSourceLeaves => {
                    self.resolve_source_linked_effect(
                        &item,
                        SourceLinkedEffectKind::RemoveAbilities,
                    );
                }
                Instruction::GrantTargetHexproofUntilSourceLeaves => {
                    self.resolve_source_linked_effect(&item, SourceLinkedEffectKind::GrantHexproof);
                }
                Instruction::PreventTargetDamageUntilSourceLeaves => {
                    self.resolve_source_linked_effect(
                        &item,
                        SourceLinkedEffectKind::PreventDamageBy,
                    );
                }
                Instruction::PreventTargetUntapWhileSourceControlled => {
                    self.resolve_source_linked_effect(&item, SourceLinkedEffectKind::PreventUntap);
                }
                Instruction::Tap | Instruction::Untap => {
                    if instruction == Instruction::Untap {
                        let ids: Vec<_> = item
                            .targets
                            .iter()
                            .filter(|bound| self.same_target_object(**bound))
                            .filter_map(|bound| match bound.target {
                                Target::Card(id) => Some(id),
                                _ => None,
                            })
                            .collect();
                        self.untap_objects(&ids, untap_rules::UntapCause::Effect);
                    } else if let [bound] = item.targets.as_slice() {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            self.tap_objects(&[id]);
                        }
                    } else {
                        let ids: Vec<_> = item
                            .targets
                            .iter()
                            .filter_map(|bound| {
                                if self.same_target_object(*bound)
                                    && let Target::Card(id) = bound.target
                                {
                                    Some(id)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        self.tap_objects(&ids);
                    }
                }
                Instruction::CounterPermanentExileCast => self.counter_permanent_exile_cast(&item),
                Instruction::CounterAbilityAndRemoveSourceAbilities => {
                    self.counter_ability_and_remove_source_abilities(&item);
                }
                Instruction::RemoveAbilitiesThenDestroyCreaturesManaAtMost(amount) => {
                    self.remove_abilities_then_destroy_creatures_mana_at_most(&item, amount);
                }
                Instruction::Counter => {
                    for bound in &item.targets {
                        if let Some(id) = self.bound_stack_item(bound).map(|s| s.id) {
                            self.counter_stack_item(id, false);
                        }
                    }
                }
                Instruction::Suspect { subject } => self.suspect_subject(&item, subject),
                Instruction::UnsuspectCreatures { filter } => {
                    self.unsuspect_creatures(&item, filter)
                }
                Instruction::ReturnGrantedSourceWithSuspect { tapped } => {
                    self.return_granted_source_suspected(&item, tapped)
                }
                Instruction::ReturnToBattlefieldSuspected => {
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|b| self.same_target_object(*b))
                        .filter_map(|b| match b.target {
                            Target::Card(id) => {
                                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                                movement.controller = item.controller;
                                movement.suspect_after = true;
                                Some(movement)
                            }
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToBattlefieldWithRole { token } => {
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => {
                                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                                movement.controller = item.controller;
                                movement.role_after = Some((token, item.controller));
                                Some(movement)
                            }
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToBattlefieldWithCounter(counter) => {
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => {
                                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                                movement.controller = item.controller;
                                movement.effect_counter = Some((counter, 1));
                                Some(movement)
                            }
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToBattlefieldWithAttackTrigger(ability) => {
                    self.return_target_with_attack_trigger(&item, ability);
                }
                Instruction::ReturnToBattlefieldUnderControl => {
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => {
                                let mut movement = self.entry_move(id, Zone::Battlefield, false);
                                movement.controller = item.controller;
                                Some(movement)
                            }
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToBattlefieldTapped => {
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => Some(self.entry_move(id, Zone::Battlefield, true)),
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToHandOrSkip { skip } => {
                    self.return_targets_to_hand_or_skip(&item, skip);
                }
                Instruction::ReturnTargetsCreateNextUpkeep { token } => {
                    self.return_for_upkeep_tokens(&item, token)
                }
                Instruction::ResolveDelayedTokenCreation => {
                    if let StackKind::DelayedTokenCreation { token, amount } = item.kind {
                        self.create_tokens(item.controller, token, u128::from(amount), false);
                    }
                }
                Instruction::ReturnAllTargetsToHand => {
                    // One printed movement shared by multiple targeting clauses.
                    // Preserve their independently validated bindings while moving
                    // an object chosen in multiple groups only once (CR115.3).
                    let mut targets: Vec<BoundTarget> = Vec::new();
                    for bound in &item.targets {
                        if bound.object.is_some()
                            && !targets.iter().any(|prior| prior.object == bound.object)
                        {
                            targets.push(*bound);
                        }
                    }
                    self.move_target_batch(&targets, Zone::Hand);
                }
                Instruction::ReturnCreaturesWithoutSubtype(subtype) => {
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && self.is_creature(*id as CardId)
                                && !self.has_subtype(*id as CardId, subtype)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Hand);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::ReturnToHandDrawForSubtype(subtype) => {
                    // Capture the legal target before changing its incarnation.
                    // The subtype condition is independent of a successful move.
                    let draws = item
                        .targets
                        .iter()
                        .filter(|&&bound| {
                            self.same_target_object(bound)
                                && match bound.target {
                                    Target::Card(id) => self.has_subtype(id, subtype),
                                    _ => false,
                                }
                        })
                        .count();
                    self.move_target_batch(&item.targets, Zone::Hand);
                    self.draw_many(item.controller, draws as u128);
                }
                Instruction::Exile
                | Instruction::Destroy
                | Instruction::ReturnToHand
                | Instruction::ReturnToBattlefield => {
                    let zone = if instruction == Instruction::Exile {
                        Zone::Exile
                    } else if instruction == Instruction::ReturnToHand {
                        Zone::Hand
                    } else if instruction == Instruction::ReturnToBattlefield {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    };
                    if instruction == Instruction::Destroy {
                        let targets: Vec<_> = item
                            .targets
                            .iter()
                            .copied()
                            .filter(|bound| match bound.target {
                                Target::Card(id) => !self.effect_destruction_prevented(id),
                                _ => true,
                            })
                            .collect();
                        self.move_target_batch(&targets, zone);
                    } else {
                        self.move_target_batch(&item.targets, zone);
                    }
                }
            }
        }
    }
}
