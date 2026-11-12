//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;
use crate::cards::{
    CastFilter, CastTarget, CreatureCastQualifier, StaticAbilityFragment, TriggerLimit,
    TurnCallbackEvent,
};

/// Effective characteristics of the committed spell face at the cast event.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
pub(super) struct SpellCastTraits {
    subtypes: GrantedSubtypes,
    keywords: Keywords,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellCastTraits {
    pub(super) fn matches_qualifier(self, qualifier: CreatureCastQualifier, card: Card) -> bool {
        qualifier.matches(
            card,
            || self.keywords,
            |subtype| self.subtypes.has(card, subtype),
        )
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TargetCardFacts {
    pub object: ObjectRef,
    pub group: u16,
    pub controller: Player,
    pub mana_value: u64,
    pub attacking: bool,
    pub damaged: bool,
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
    pub shuffle: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub power_difference: Option<Box<effect_choices::PowerDifferenceView>>,
    pub chosen_color: Option<crate::cards::ChoiceColor>,
    pub chosen_card_name: Option<crate::CardName>,
    pub chosen_creature_type: Option<crate::cards::CreatureType>,
    pub target_counter_removal: Option<Box<TargetCounterRemovalView>>,
    pub linked_copy: Option<Box<linked_copies::LinkedCopyCapture>>,
    pub blight_copy: Option<Box<CopyRecipe>>,
    pub cascade: Option<Box<cascade::CascadeState>>,
    #[allow(clippy::box_collection)]
    pub target_card_facts: Option<Box<Vec<TargetCardFacts>>>,
    pub exile_continuation: Option<Box<return_links::ExileContinuation>>,
    /// A count owned by the current repeating instruction until it advances.
    pub repeated_instruction_remaining: Option<u32>,
    pub connive: Option<Box<super::connive::ConniveSequence>>,
    pub proliferate: Option<Box<super::proliferate::ProliferateSequence>>,
    pub explore: Option<Box<explore::ExploreSequence>>,
    pub manifest_sequence: Option<Box<manifest::ManifestSequence>>,
    pub evidence: Option<Box<evidence::EvidenceSelection>>,
    pub item: SpellView,
    pub mana_payment: Option<Box<resolution_mana::ResolutionPayment>>,
    pub forage: Option<Box<forage::ForageSelection>>,
    pub repeated_life_choice: Option<Box<effect_choices::RepeatedLifeChoice>>,
    pub target_library: Option<Box<target_library::TargetLibraryChoice>>,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub target_group: u16,
    /// The instant or sorcery's original target bindings all named one exact card.
    /// Resolution may discard illegal bindings, which cannot qualify a multi-target spell.
    pub original_single_spell_card_target: bool,
    pub moved_count: Option<u64>,
    exiled_library_land: bool,
    exiled_library_nonland: bool,
    pub remembered_draws: Option<u64>,
    pub remembered_damage_players: Option<u8>,
    pub remembered_objects: Option<SharedVec<ObjectRef>>,
    pub sacrificed_power: Option<Box<i128>>,
    pub sacrificed_mana_value: Option<u64>,
    pub discard_after: Option<discard_events::DiscardAfter>,
    pub optional_wheel: Option<OptionalWheel>,
    pub color_sequence: Option<Box<super::color_sequences::ColorCounterSequence>>,
    pub discards: [usize; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub milled: Vec<ObjectRef>,
    pub(super) discarded_card: Option<ObjectRef>,
    pub milled_selection: Option<Box<super::milled_selection::MilledSelection>>,
    pub library: Option<Box<LibraryChoice>>,
    pub selection: Option<Box<super::library_selection::LibrarySelection>>,
    pub return_attachment: Option<Box<super::return_attachment::ReturnAttachment>>,
    pub search: Option<Box<super::search::SearchChoice>>,
    pub deferred_search_shuffle: bool,
    pub search_result: Option<Box<super::search::SearchResult>>,
    /// The target's controller and name before exile, awaiting any linked entries.
    pub exile_name_search: Option<Box<(Player, &'static str)>>,
    pub optional_searches: Option<Box<super::search::OptionalSearchSequence>>,
    pub choice: Option<Box<super::effect_choices::EffectChoice>>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibraryChoice {
    pub surveil: bool,
    remember_top: bool,
    pub return_within_life_gained: bool,
    pub return_noncreature_nonland: bool,
    pub remaining: Vec<CardId>,
    pub top: Vec<CardId>,
    pub bottom: Vec<CardId>,
    pub graveyard: Vec<CardId>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn departed_source_in_graveyard<'a>(
        &'a self,
        item: &'a SpellView,
    ) -> impl Iterator<Item = CardId> + 'a {
        item.source_move
            .iter()
            .filter(|event| event.destination == Zone::Graveyard)
            .flat_map(|event| event.objects_after())
            .filter(|&object| {
                self.object_ref(object.card) == object
                    && self.state.objects[object.card as usize].zone == Zone::Graveyard
                    && self.state.objects[object.card as usize].is_card()
            })
            .map(|object| object.card)
    }
    fn exile_departed_source(&mut self, item: &SpellView) -> bool {
        let ids = self.departed_source_in_graveyard(item).collect::<Vec<_>>();
        if ids.is_empty() {
            return false;
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let moved = events
            .iter()
            .any(|event| event.moved && event.destination == Zone::Exile);
        self.finish_zone_events(&events, &[]);
        moved
    }

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

    fn subject_has_subtype(&self, item: &SpellView, subject: Subject, subtype: &str) -> bool {
        let id = match subject {
            Subject::Entering => self.entering_subject(item),
            Subject::Source => item.source.and_then(|source| {
                (self.on_battlefield(source.id) && self.object_ref(source.id) == source.object)
                    .then_some(source.id)
            }),
            Subject::Attached => {
                if let Some(event) = &item.attachment_trigger {
                    (self.on_battlefield(event.recipient.card)
                        && self.object_ref(event.recipient.card) == event.recipient)
                        .then_some(event.recipient.card)
                } else {
                    self.attached_permanent(item)
                }
            }
            Subject::Targets => match item.targets.as_slice() {
                [bound] if self.same_target_object(*bound) => match bound.target {
                    Target::Card(id) => Some(id),
                    _ => None,
                },
                _ => None,
            },
        };
        id.is_some_and(|id| self.on_battlefield(id) && self.has_subtype(id, subtype))
    }

    pub(super) fn start_library_arrangement(
        &mut self,
        player: Player,
        amount: usize,
        surveil: bool,
    ) -> bool {
        let remaining: Vec<_> = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .filter(|&&id| self.state.objects[id as usize].is_card())
            .take(amount)
            .copied()
            .collect();
        if remaining.is_empty() {
            if amount != 0 {
                self.scry_or_surveil_event(player);
            }
            return false;
        }
        self.state.resolving.as_mut().unwrap().library = Some(Box::new(LibraryChoice {
            surveil,
            remember_top: false,
            return_within_life_gained: false,
            return_noncreature_nonland: false,
            remaining,
            top: Vec::new(),
            bottom: Vec::new(),
            graveyard: Vec::new(),
        }));
        self.state.pending = Pending::EffectLibrary { player };
        true
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
        if choice.remember_top {
            self.state.resolving.as_mut().unwrap().moved_count =
                Some(u64::try_from(choice.top.len()).capacity("surveil kept count exhausted"));
        }
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
            for (index, ability) in source.trigger_templates_from(grant.as_ref()) {
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
        let amount = life_loss::replacement_amount(self, player, amount);
        self.lose_life_unobserved(player, amount);
        self.life_lost_event(player, amount);
    }

    pub(super) fn lose_life_simultaneously(&mut self, amounts: [i128; 2]) {
        let amounts = [Player::P0, Player::P1]
            .map(|player| life_loss::replacement_amount(self, player, amounts[player.index()]));
        let order = [self.state.active, self.state.active.other()];
        for player in order {
            self.lose_life_unobserved(player, amounts[player.index()]);
        }
        for player in order {
            self.life_lost_event(player, amounts[player.index()]);
        }
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
                self.lose_life_unobserved(player, receipt.lost);
                self.life_lost_event(player, receipt.lost);
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
            let entering = self
                .state
                .entry_batch
                .as_ref()
                .map_or(&[][..], |batch| batch.moves.as_slice());
            let count = usize::try_from(amount).unwrap_or(usize::MAX);
            let library = self.state.players[player.index()]
                .library
                .iter()
                .rev()
                .copied()
                .filter(|&id| self.state.objects[id as usize].is_card())
                .map(|id| self.object_ref(id));
            let cards = entries::life_replacement_exile_cards(library, count, entering);
            if let Some(Some(receipt)) = &frozen
                && receipt.exiled.as_ref() != cards.as_ref()
            {
                self.fail_frozen_mana_replay();
                return;
            }
            if let Some(cards) = cards {
                let ids: Vec<_> = cards.iter().map(|object| object.card).collect();
                self.record_cast_mana_life_payment(player, amount, 0, Some(cards));
                if !ids.is_empty() {
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    self.finish_zone_events(&events, &[]);
                }
                return;
            }
        }
        let lost = life_loss::replacement_amount(self, player, amount);
        self.record_cast_mana_life_payment(player, amount, lost, None);
        self.lose_life_unobserved(player, lost);
        self.life_lost_event(player, lost);
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
                self.has_trigger(id, |a| {
                    matches!(
                        a.condition.event(),
                        TriggerCondition::LifeLost { .. }
                            | TriggerCondition::LifeGainedOrLostOwnTurn
                    )
                })
            })
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.trigger_templates() {
                let matched = match ability.condition.event() {
                    TriggerCondition::LifeLost { relation } => match relation {
                        CastRelation::You => source.controller == player,
                        CastRelation::Opponent => source.controller != player,
                        CastRelation::Any => true,
                    },
                    TriggerCondition::LifeGainedOrLostOwnTurn => {
                        source.controller == player && active == player
                    }
                    _ => false,
                };
                if matched && self.queue_trigger(&source, index) {
                    self.state.pending_triggers.last_mut().unwrap().life_event =
                        Some(LifeTriggerView { player, amount });
                }
            }
        }
    }

    pub(super) fn has_counter_batch_observers(&self) -> bool {
        self.state.counter_sources.iter().any(|&id| {
            self.has_trigger(id, |a| {
                matches!(
                    a.condition.event(),
                    TriggerCondition::CounterBatchPlacedByYou { .. }
                        | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                        | TriggerCondition::CreatureCountersPutByYou
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
                self.has_trigger(id, |a| {
                    matches!(
                        a.condition.event(),
                        TriggerCondition::CounterBatchPlacedByYou { .. }
                            | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                            | TriggerCondition::CreatureCountersPutByYou
                    )
                })
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.trigger_templates() {
                let condition = ability.condition.event();
                match condition {
                    TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                    | TriggerCondition::CreatureCountersPutByYou => {
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
                                && match condition {
                                    TriggerCondition::CounterPlacedByYouOnControlledSubtypes {
                                        subtypes,
                                    } => subtypes
                                        .iter()
                                        .any(|subtype| self.has_subtype(object.card, subtype)),
                                    TriggerCondition::CreatureCountersPutByYou => {
                                        self.is_creature(object.card)
                                    }
                                    _ => unreachable!(),
                                }
                                && self.queue_trigger(&source, index)
                            {
                                self.state
                                    .pending_triggers
                                    .last_mut()
                                    .unwrap()
                                    .refer_objects([object]);
                            }
                        }
                    }
                    TriggerCondition::CounterBatchPlacedByYou { kind, filter } => {
                        let matching: Vec<_> = placements
                            .clone()
                            .filter(|&(object, amount, placed_by)| {
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
                            })
                            .map(|(object, _, _)| object)
                            .collect();
                        if !matching.is_empty() && self.queue_trigger(&source, index) {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .refer_objects(matching);
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
        self.record_counter_placement(
            object,
            placed_by,
            kind == CounterKind::PlusOne,
            kind == CounterKind::Lore,
        );
        if self.has_trigger(object.card, |a| {
            matches!(a.condition.event(), TriggerCondition::CounterThresholdCrossed { kind: expected, .. } if expected == kind)
        }) {
            let after = self.counter_count(object.card, kind);
            let before = after.checked_sub(amount).expect("committed placement amount");
            let source = self.object_view(object.card);
            for (index, ability) in source.trigger_templates() {
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
                for (index, ability) in source.trigger_templates() {
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
                    if qualifies && self.queue_trigger(&source, index) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .set_event_object(EventObjectView {
                                object,
                                facts: EventObjectFacts::CountersPlaced { amount },
                                semantic_objects: None,
                                conniver_creature: None,
                                subtype_lki: None,
                            });
                    }
                }
            }
        }
        if !self.has_trigger(object.card, |a| matches!(a.condition.event(), TriggerCondition::SelfCountersAdded { kind: expected, .. } if expected == kind)) { return; }
        let source = self.object_view(object.card);
        for (index, ability) in source.trigger_templates() {
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

    pub(super) fn commit_life_gain(&mut self, player: Player, amount: i128) {
        if amount <= 0 {
            return;
        }
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
            for (index, ability) in source.trigger_templates() {
                let matched = match ability.condition.event() {
                    TriggerCondition::LifeGained {
                        first_only,
                        own_turn,
                    } => (!first_only || first) && (!own_turn || self.state.active == player),
                    TriggerCondition::LifeGainedOrLostOwnTurn => self.state.active == player,
                    _ => false,
                };
                if matched && self.queue_trigger(&source, index) {
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

    fn exact_source_in_zone(&self, item: &SpellView, zone: TargetZone) -> Option<CardId> {
        let source = item.source?;
        let zone = match zone {
            TargetZone::Battlefield => Zone::Battlefield,
            TargetZone::Graveyard => Zone::Graveyard,
        };
        (self.object_ref(source.id) == source.object
            && self.state.objects[source.id as usize].zone == zone)
            .then_some(source.id)
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
        self.controlled_creature_death_triggers(event, death_sources);
        self.forget_self_grants(event);
        if event.before.zone == Zone::Battlefield {
            self.battlefield_departure_triggers(event);
            // A self-departure trigger needs only its frozen pre-event source.
            // This also covers exile/bounce paths which do not collect death observers.
            let source = &event.before;
            for (index, ability) in source.trigger_templates() {
                if ability.condition.event() == TriggerCondition::SelfLeaves {
                    self.queue_zone_trigger(source, index, event);
                }
            }
        }
        if event.before.zone == Zone::Battlefield {
            // Departure triggers look back at the entire pre-event battlefield;
            // simultaneous departures still see each other exactly once.
            for source in death_sources {
                for (index, ability) in source.trigger_templates() {
                    if event.destination != Zone::Graveyard
                        && !matches!(
                            ability.condition.event(),
                            TriggerCondition::Sacrificed { .. }
                                | TriggerCondition::SacrificedByOpponent { .. }
                                | TriggerCondition::SacrificedByAny { .. }
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
                                && self.state.permanent_damage_history.as_ref().is_some_and(
                                    |rows| {
                                        rows.iter().any(|row| {
                                            row.was_creature
                                                && row.view.source == source.reference
                                                && row.view.recipient == event.before.reference
                                        })
                                    },
                                )
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
                        condition @ (TriggerCondition::CreatureDiesWithCounters { relation }
                        | TriggerCondition::CreatureDiesWithCounter {
                            relation, ..
                        }) => {
                            let counters = &event.before.counters;
                            let matches_counters = match condition {
                                TriggerCondition::CreatureDiesWithCounter { kind, .. } => {
                                    counters.get(kind) > 0
                                }
                                _ => counters.has_any(),
                            };
                            event.before.is_creature()
                                && matches_counters
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
                        condition @ (TriggerCondition::OtherControlledPermanentWithSubtypeDies { subtypes }
                        | TriggerCondition::ControlledPermanentWithSubtypeDiesAny(subtypes)) => {
                            source.controller == event.before.controller
                                && (matches!(condition, TriggerCondition::ControlledPermanentWithSubtypeDiesAny(_))
                                    || source.reference != event.before.reference)
                                && subtypes.iter().any(|subtype| event.before.has_subtype(subtype))
                        }
                        TriggerCondition::ControlledLegendaryCreatureDies => {
                            source.controller == event.before.controller
                                && event.before.is_creature()
                                && event.before.is_legendary()
                        }
                        TriggerCondition::ControlledPermanentWithSubtypeDies(subtype) => {
                            source.controller == event.before.controller
                                && event.before.has_subtype(subtype)
                        }
                        TriggerCondition::Sacrificed { filter: f, .. }
                        | TriggerCondition::SacrificedByOpponent { filter: f }
                        | TriggerCondition::SacrificedByAny { filter: f } => {
                            let (player, own_turn) = match ability.condition.event() {
                                TriggerCondition::Sacrificed { own_turn, .. } => {
                                    (Some(source.controller), own_turn)
                                }
                                TriggerCondition::SacrificedByOpponent { .. } => {
                                    (Some(source.controller.other()), false)
                                }
                                TriggerCondition::SacrificedByAny { .. } => (None, false),
                                _ => unreachable!(),
                            };
                            event
                                .sacrificed_by
                                .is_some_and(|actor| player.is_none_or(|p| p == actor))
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
                        condition @ (TriggerCondition::CreatureDiesWithMaximumStat { relation, maximum }
                        | TriggerCondition::CreatureDiesWithMaximumToughness { relation, maximum }) => {
                            event.before.is_creature()
                                && (event.before.toughness <= i128::from(maximum)
                                    || (matches!(condition, TriggerCondition::CreatureDiesWithMaximumStat { .. })
                                        && event.before.power <= i128::from(maximum)))
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
                            let allowed_kinds = if source.reference == event.before.reference {
                                kinds & !exclude_source_kinds
                            } else {
                                kinds
                            };
                            related && event.before.kinds & allowed_kinds != 0
                        }
                        _ => false,
                    };
                    if matched && self.queue_zone_trigger(source, index, event) {
                        self.capture_battlefield_departure_facts(event, ability.program);
                        if let TriggerCondition::ReturnDeath(filter) = ability.condition.event() {
                            self.capture_return_death(event, filter);
                        }
                        let return_tapped = ability.program.iter().any(|instruction| {
                            matches!(
                                instruction,
                                Instruction::ReturnEventCardToBattlefieldTapped
                                    | Instruction::ReturnEventCardToBattlefieldTappedUnderControl
                            )
                        });
                        if return_tapped
                            || ability
                                .program
                                .contains(&Instruction::ReturnEventCardToHand)
                        {
                            self.state.pending_triggers.last_mut().unwrap().return_link =
                                Some(Box::new(ReturnLinkView::from_event(
                                    event,
                                    event.before.owner,
                                    None,
                                    return_tapped,
                                    false,
                                )));
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
            for (index, ability) in source.trigger_templates() {
                let matched = match ability.condition.event() {
                    TriggerCondition::SelfDiesWithCounters
                    | TriggerCondition::InZone { .. }
                    | TriggerCondition::ManaSpentBelowValue(_)
                    | TriggerCondition::ManaSpent { .. }
                    | TriggerCondition::TreasureManaSpent(_)
                    | TriggerCondition::Granted(_)
                    | TriggerCondition::OncePerTurn(_)
                    | TriggerCondition::OncePerIncarnation(_)
                    | TriggerCondition::DuringOpponentsTurn(_)
                    | TriggerCondition::When { .. }
                    | TriggerCondition::InterveningIf { .. } => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::CastFromHand(_)
                    | TriggerCondition::CastFromExile(_)
                    | TriggerCondition::PlayFromOutsideOwnHand(_) => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::SelfEntersIfCast => same && event.was_cast,
                    TriggerCondition::SelfEntersIfCastByController => {
                        same && event.was_cast
                            && event
                                .cast_event
                                .as_ref()
                                .is_some_and(|cast| cast.controller == source.controller)
                    }
                    TriggerCondition::ControlledPermanentEntersFrom(zone) => {
                        yours && event.before.zone == zone
                    }
                    TriggerCondition::SelfEntersFromGraveyard => {
                        same && event.before.zone == Zone::Graveyard
                    }
                    TriggerCondition::SelfEntersIfCastFromYourGraveyard => {
                        same && event.was_cast
                            && event.cast_origin == Some(Zone::Graveyard)
                            && event.before.owner == entered.controller
                    }
                    TriggerCondition::SelfEntersWithWebSlinging => {
                        same && event
                            .cast_event
                            .as_ref()
                            .is_some_and(|cast| cast.web_slinging_return_mana_value.is_some())
                    }
                    TriggerCondition::SelfEvokedEnters => same && event.evoked,
                    TriggerCondition::SelfEntersManaSpent { color, minimum } => {
                        same && event.cast_event.as_ref().is_some_and(|e| {
                            e.payment
                                .get(usize::from(color))
                                .is_some_and(|n| *n >= minimum)
                        })
                    }
                    TriggerCondition::SelfNontokenEnters => same && !entered.is_token(),
                    TriggerCondition::SelfOffspring => same && event.offspring_paid,
                    TriggerCondition::SelfKicked => same && event.kicked,
                    TriggerCondition::SelfGiftEnters => same && event.gift.is_some(),
                    TriggerCondition::SelfBargained => same && event.bargained,
                    TriggerCondition::SelfEvidence => same && event.evidence_collected,
                    TriggerCondition::EvidenceCollected
                    | TriggerCondition::ScryOrSurveil
                    | TriggerCondition::DiscoveredYou
                    | TriggerCondition::Proliferated(_)
                    | TriggerCondition::Chapter { .. } => false,
                    TriggerCondition::SelfEnters
                    | TriggerCondition::SelfEntersOrBecomesTapped
                    | TriggerCondition::SelfEntersOrTransforms
                    | TriggerCondition::SelfEntersOrTurnsFaceUp
                    | TriggerCondition::SelfEntersOrAttacks
                    | TriggerCondition::SelfEntersOrAttacksWhileSaddled
                    | TriggerCondition::SelfEntersOrDies => same,
                    TriggerCondition::CreaturesFromGraveyardEnterYou { .. }
                    | TriggerCondition::CreaturesEnterYouWithoutManaSpent
                    | TriggerCondition::NontokenCreaturesEnterFromExile
                    | TriggerCondition::SelfBecomesPlotted
                    | TriggerCondition::SelfTransforms
                    | TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    | TriggerCondition::OpponentTargetsControlledCreatureOrSpell
                    | TriggerCondition::OpponentTargetsControlledCreature
                    | TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent
                    | TriggerCondition::CommittedCrime
                    | TriggerCondition::TokensEnterYou
                    | TriggerCondition::FilteredEntersBatchYou { .. }
                    | TriggerCondition::TokensEnterOpponent
                    | TriggerCondition::CreatureAbilityActivated
                    | TriggerCondition::AbilityActivatedTargetingCreatureOrPlayer
                    | TriggerCondition::NonmanaAbilityActivated
                    | TriggerCondition::ExhaustAbilityActivated
                    | TriggerCondition::DelayedExhaustAbilityActivated
                    | TriggerCondition::DelayedTargetEvent
                    | TriggerCondition::DelayedTargetControlLostWhileAttached
                    | TriggerCondition::OpponentTakesPermanent
                    | TriggerCondition::ControlledAuraAttachesToOpponentNonland
                    | TriggerCondition::SelfBecomesAttachedToCreature
                    | TriggerCondition::LoyaltyAbilityActivated
                    | TriggerCondition::LoyaltyAbilityActivatedRemoving(_)
                    | TriggerCondition::OpponentLoyaltyAbilityActivated
                    | TriggerCondition::CardsLeaveGraveyard
                    | TriggerCondition::CardsLeaveGraveyardKinds(_)
                    | TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
                    | TriggerCondition::CreatureCardLeavesYourGraveyard
                    | TriggerCondition::PermanentCardsEnterYourGraveyard
                    | TriggerCondition::CreatureCardsEnterYourGraveyard
                    | TriggerCondition::LandCardsEnterYourGraveyardFromLibrary
                    | TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
                    | TriggerCondition::SelfExiledWhileCrafting
                    | TriggerCondition::CreatureExploresYou
                    | TriggerCondition::SelfTurnedFaceUp
                    | TriggerCondition::PermanentYouControlTurnedFaceUp
                    | TriggerCondition::SelfOrControlledCreatureTurnedFaceUp
                    | TriggerCondition::YouTurnPermanentFaceUp
                    | TriggerCondition::YouManifestDread
                    | TriggerCondition::CreatureExploresCardYou { .. }
                    | TriggerCondition::Foraged
                    | TriggerCondition::GiftGiven
                    | TriggerCondition::BendsYou
                    | TriggerCondition::OwnAttackerAbilityTriggered
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Reflexive
                    | TriggerCondition::DelayedSpellCastAny
                    | TriggerCondition::DelayedSpellCastSubtype(_)
                    | TriggerCondition::DelayedSpellCastCreature
                    | TriggerCondition::DelayedInstantSorceryCastManaValueAtMost { .. }
                    | TriggerCondition::DelayedSpellCastSourcePower
                    | TriggerCondition::Ward
                    | TriggerCondition::SelfTargeted { .. }
                    | TriggerCondition::FirstTargetedByController
                    | TriggerCondition::CardDrawn { .. }
                    | TriggerCondition::CaseSolved(_)
                    | TriggerCondition::LibrarySearched { .. }
                    | TriggerCondition::SpellCastUsingSourceMana(_)
                    | TriggerCondition::FirstSpellKindOrOtherCreatureType { .. }
                    | TriggerCondition::BecomesClassLevel(_)
                    | TriggerCondition::SelfDiscarded
                    | TriggerCondition::SelfCycled
                    | TriggerCondition::Discarded { .. }
                    | TriggerCondition::DiscardedKinds { .. }
                    | TriggerCondition::ControlledCreatureCombatDamageOwnTurn
                    | TriggerCondition::SourceDealtDamage
                    | TriggerCondition::SourceDealtDamageAtLeast(_)
                    | TriggerCondition::SourceDealtDamageToCreatureSubtype(_)
                    | TriggerCondition::SourceDealtCombatDamage
                    | TriggerCondition::SourceCombatDamagePlayerOrPlaneswalker
                    | TriggerCondition::AttachedDealtCombatDamage
                    | TriggerCondition::AttachedDealtCombatDamageToPlayer
                    | TriggerCondition::DamagedCreatureDies
                    | TriggerCondition::StateCounterAtLeast { .. }
                    | TriggerCondition::OwnUntapStepUntaps
                    | TriggerCondition::CounterThresholdCrossed { .. }
                    | TriggerCondition::PlayerDamaged { .. }
                    | TriggerCondition::AttachedPlayerDamaged
                    | TriggerCondition::CombatDamageToController
                    | TriggerCondition::CreatureDamaged { .. }
                    | TriggerCondition::ExcessNoncombatDamageToCreatures { .. }
                    | TriggerCondition::SelfDamaged
                    | TriggerCondition::SelfDamagedAtLeast(_)
                    | TriggerCondition::AttachedCreatureDamaged
                    | TriggerCondition::ControlledSpellDamagesSoleCreatureTarget
                    | TriggerCondition::ControlledNoncombatDamageMatchesCreatureToughness
                    | TriggerCondition::SelfDies
                    | TriggerCondition::SelfNontokenDies
                    | TriggerCondition::SelfCreatureDies
                    | TriggerCondition::SelfLeaves
                    | TriggerCondition::PermanentLeaves(_)
                    | TriggerCondition::BattlefieldDeparture(_)
                    | TriggerCondition::BattlefieldDepartureBatch(_)
                    | TriggerCondition::SelfOrControlledPermanentDies(_)
                    | TriggerCondition::OpponentHighestPowerCreatureDies
                    | TriggerCondition::ChosenCreatureLeaves
                    | TriggerCondition::ChosenPlayerLosesGame
                    | TriggerCondition::ReturnDeath(_)
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::CreatureDiesWithCounters { .. }
                    | TriggerCondition::CreatureDiesWithCounter { .. }
                    | TriggerCondition::OtherControlledPermanentWithSubtypeDies { .. }
                    | TriggerCondition::ControlledPermanentWithSubtypeDies(_)
                    | TriggerCondition::ControlledPermanentWithSubtypeDiesAny(_)
                    | TriggerCondition::ControlledLegendaryCreatureDies
                    | TriggerCondition::AttachedCreatureDies
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesWithMaximumStat { .. }
                    | TriggerCondition::CreatureDiesWithMaximumToughness { .. }
                    | TriggerCondition::CreatureDiesBatch { .. }
                    | TriggerCondition::PermanentDiesBatch { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::CreatureDiesAttackState { .. }
                    | TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::SacrificedByOpponent { .. }
                    | TriggerCondition::SacrificedByAny { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::LifeLost { .. }
                    | TriggerCondition::LifeGainedOrLostOwnTurn
                    | TriggerCondition::SelfCountersAdded { .. }
                    | TriggerCondition::CreatureAttacksYou { .. }
                    | TriggerCondition::CreaturesAttackYou
                    | TriggerCondition::CreaturesAttackYouOrPlaneswalkersAtLeast(_)
                    | TriggerCondition::PlayerAttacksWithAtLeast(_)
                    | TriggerCondition::OpponentAttacked
                    | TriggerCondition::OtherCreatureCountersAdded { .. }
                    | TriggerCondition::OtherCreatureCounterPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYou { .. }
                    | TriggerCondition::ControlledCreatureCountersPlaced { .. }
                    | TriggerCondition::CounterBatchPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYouOnControlledSubtypes { .. }
                    | TriggerCondition::CreatureCountersPutByYou
                    | TriggerCondition::AttackDeclared { .. }
                    | TriggerCondition::CreaturesAttackDeclared { .. }
                    | TriggerCondition::FirstAttackWithTotalPowerAtLeast(_)
                    | TriggerCondition::AttackPlayer(_)
                    | TriggerCondition::AttackPlayerExcludingSubtype(_)
                    | TriggerCondition::AttackPlayerWithPowerAtLeast(_)
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SelfAttacksDefendingPlayer
                    | TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype(_)
                    | TriggerCondition::SelfAttacksPlayerWithMostLife
                    | TriggerCondition::SelfAttacksIfEquipped
                    | TriggerCondition::SelfFirstAttack
                    | TriggerCondition::SelfAttacksAlone
                    | TriggerCondition::SelfAttacksWithOthers
                    | TriggerCondition::ControlledCreatureAttacksAlone
                    | TriggerCondition::ControlledCreatureAttacksOrBlocks
                    | TriggerCondition::DelayedCreatureAttacksAlone
                    | TriggerCondition::DelayedCreatureAttacksSubtypeAny(_)
                    | TriggerCondition::SelfAttacksOrBlocks
                    | TriggerCondition::SelfBlocksCreatureWithKeyword(_)
                    | TriggerCondition::SelfBlocksOrBecomesBlockedByCreature
                    | TriggerCondition::SelfBecomesBlocked
                    | TriggerCondition::ControlledSubtypeBecomesBlocked(_)
                    | TriggerCondition::ControlledCreatureBecomesBlocked
                    | TriggerCondition::ControlledCreatureAttacks
                    | TriggerCondition::ControlledSuspectedCreaturesAttack
                    | TriggerCondition::CrewedVehicleAttacks
                    | TriggerCondition::ControlledSubtypeAttackDeclaration(_)
                    | TriggerCondition::ControlledSubtypeAttacks { .. }
                    | TriggerCondition::ControlledSubtypeAttacksAny(_)
                    | TriggerCondition::ControlledSubtypeAttackAtLeast { .. }
                    | TriggerCondition::ControlledCreatureAttacksWithKeyword(_)
                    | TriggerCondition::ControlledCreatureWithKeywordAttackDeclaration(_)
                    | TriggerCondition::ControlledCreatureAttacksWithCounter(_)
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
                    | TriggerCondition::AttachedAttacks
                    | TriggerCondition::AttachedAttacksOrBlocks
                    | TriggerCondition::AttachedCreatureBecomesTapped
                    | TriggerCondition::AttachedBecomesUntapped
                    | TriggerCondition::SelfBecomesTapped
                    | TriggerCondition::ControlledCreaturesBecomeTapped
                    | TriggerCondition::ControlledPermanentsOfSubtypeTapped { .. }
                    | TriggerCondition::ControlledCreatureFirstTapOwnTurn
                    | TriggerCondition::OpponentCreatureTappedByYou { .. }
                    | TriggerCondition::SelfCast
                    | TriggerCondition::SpellCast { .. }
                    | TriggerCondition::SpellCastOrLandPlayed
                    | TriggerCondition::ModalSpellCastByYou
                    | TriggerCondition::ManaSpentForSpell
                    | TriggerCondition::Expend(_)
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
                            && self.entry_predicate_matches(predicate, source, &entered)
                    }
                    TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
                        entry,
                        other_on_entry,
                        ..
                    } => {
                        yours
                            && (!other_on_entry || !same)
                            && self.entry_predicate_matches(entry, source, &entered)
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
                    TriggerCondition::EnchantmentEntersYou | TriggerCondition::Eerie => {
                        yours && entered.kinds & 32 != 0
                    }
                    TriggerCondition::FaceDownPermanentEntersOrTurnsUpYou => {
                        yours && self.is_face_down(event.after)
                    }
                    TriggerCondition::SelfDoorUnlocked(_) => false,
                    TriggerCondition::CreatureEntersYouIfCast => {
                        yours
                            && entered.is_creature()
                            && event.was_cast
                            && event
                                .cast_event
                                .as_ref()
                                .is_some_and(|cast| cast.controller == source.controller)
                    }
                    TriggerCondition::CreatureEntersYou => yours && entered.is_creature(),
                    TriggerCondition::LandEntersYou => yours && entered.kinds & 2 != 0,
                    TriggerCondition::OtherArtifactEntersYou => {
                        yours && !same && entered.kinds & 16 != 0
                    }
                };
                if matched && self.queue_zone_trigger(source, index, event) {
                    self.add_entry_trigger_multiplicity(
                        source,
                        std::slice::from_ref(&entered.reference),
                    );
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
                    self.bind_stat_subject(event.after, ability.program);
                    self.bind_entering_conniver(&entered, ability.program);
                    self.bind_entering_copy(&entered, ability.program);
                    self.bind_entering_subtype_lki(&entered, ability.program);
                    if same && source.card.x_symbols() > 0 {
                        self.state.pending_triggers.last_mut().unwrap().x = Some(event.x);
                    }
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    packet.entering_object = Some(event.after);
                    packet.evidence_collected = event.evidence_collected;
                    if matches!(
                        ability.condition.event(),
                        TriggerCondition::FilteredEntersOrTurnsFaceUpYou { .. }
                    ) && packet.event_object.is_none()
                    {
                        packet.set_event_object(EventObjectView {
                            object: event.after,
                            facts: EventObjectFacts::Controller(entered.controller),
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                    }
                }
            }
        }
    }

    pub(super) fn queue_zone_trigger(
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
        for (index, ability) in event.before.trigger_templates() {
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
        if source.additional_creature_death_triggers != 0 && event.is_creature_death() {
            let packet = self.state.pending_triggers.last().unwrap();
            let cause = packet.triggered_ability().condition.event();
            if !matches!(
                cause,
                TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::SacrificedByOpponent { .. }
                    | TriggerCondition::SacrificedByAny { .. }
            ) {
                self.add_event_trigger_multiplicity(source.additional_creature_death_triggers);
            }
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
        if event.destination == Zone::Battlefield
            && event.after == source.reference
            && let Some(recipient) = source.attached_to
            && self
                .state
                .pending_triggers
                .last()
                .unwrap()
                .program()
                .is_some_and(|(_, program)| {
                    program.contains(&Instruction::AttachedCreaturePowerDamage)
                })
        {
            self.bind_trigger_damage_actor(recipient);
        }
        if self
            .state
            .pending_triggers
            .last()
            .unwrap()
            .triggered_ability()
            .targets
            .reads_event_mana_value()
        {
            self.bind_zone_event_mana_value(event);
        }
        // The event condition names its matched object independently of any
        // effect-specific payload. Public zone moves may also find the new
        // object (CR400.7e); hidden pre-event hand/library identities are not
        // retained merely because they occurred in the implementation event.
        let packet = self.state.pending_triggers.last_mut().unwrap();
        let ability = packet.triggered_ability();
        let departure_observer =
            ability.condition.event() == TriggerCondition::CreatureCardLeavesYourGraveyard;
        if event.before.zone == Zone::Battlefield
            && event.destination == Zone::Graveyard
            && ability.program.iter().any(|instruction| {
                matches!(
                    instruction,
                    Instruction::ExileEventGraveyardCard
                        | Instruction::ExileEventGraveyardCardLinked
                )
            })
        {
            let facts = event
                .objects_after()
                .find(|&object| object != event.after)
                .map_or(
                    EventObjectFacts::Identity,
                    EventObjectFacts::GraveyardCompanion,
                );
            packet.set_event_object(EventObjectView {
                object: event.after,
                facts,
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
        }
        if (event.destination != Zone::Battlefield || departure_observer)
            && matches!(
                event.before.zone,
                Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
            )
        {
            packet.refer_objects([event.before.reference]);
        }
        if matches!(
            event.destination,
            Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
        ) {
            packet.refer_objects(event.objects_after());
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
                Some(Box::new(event.source_movement()));
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
        use granted_abilities::ACQUIRED_TRIGGER_INDEX;
        let (index, acquired) = if index >= ACQUIRED_TRIGGER_INDEX {
            let Some(acquired) = source.acquired_triggers.as_deref() else {
                return false;
            };
            (index - ACQUIRED_TRIGGER_INDEX, Some(acquired))
        } else {
            (index, None)
        };
        if let Some(grant) = event.as_ref().filter(|grant| grant.granted) {
            if acquired.is_some()
                || !source.grant_available(
                    grant.grantor,
                    grant.granting_card,
                    u16::try_from(index).expect("ability index"),
                    None,
                )
            {
                return false;
            }
        } else if !source.printed_abilities_available() {
            return false;
        }
        let definition = acquired.map_or_else(
            || {
                event
                    .as_ref()
                    .filter(|event| event.granted)
                    .map_or(source.card, |event| event.granting_card)
            },
            |grant| grant.definition,
        );
        let acquired_link_cohort = acquired.map(|grant| grant.cohort);
        let ability_origin = event.as_ref().filter(|event| event.granted).map_or_else(
            || {
                source
                    .temporary_copy
                    .as_ref()
                    .map(|view| view.active_origin)
            },
            |event| event.origin,
        );
        if acquired.is_none()
            && event.as_ref().is_none_or(|grant| !grant.granted)
            && (!entry_modes::trigger_available(source, definition, index)
                || !rooms::printed_trigger_available(source, definition, index))
        {
            return false;
        }
        let condition = definition.triggers()[index].condition;
        if condition.event() == TriggerCondition::SelfAttacksIfEquipped
            && self.source_equipment_count(source.reference).unwrap_or(0) == 0
        {
            return false;
        }
        if condition.without_incarnation_limit() == TriggerCondition::SelfDiesWithCounters
            && !source.counters.has_any()
        {
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

        if !self.trigger_time_guards_hold(condition, source)
            || !self.trigger_guards_hold(
                condition,
                source.controller,
                source.reference,
                source.tapped,
            )
        {
            return false;
        }
        if matches!(
            definition.triggers()[index].program.first(),
            Some(
                Instruction::IfOptionalAbilityUnused { .. }
                    | Instruction::MayOncePerTurn { .. }
                    | Instruction::MayCopyOncePerTurn { .. }
                    | Instruction::LookLibraryTopMayCastFreeOncePerTurn
            )
        ) && self.state.triggered_this_turn.contains(&TriggerUse {
            source: source.reference,
            ability: u16::try_from(index).expect("ability index"),
            origin: ability_origin,
            acquired_link_cohort,
            controller: Some(source.controller),
        }) {
            return false;
        }
        let trigger_limit = condition.trigger_limit();
        // An occurrence counts even if its announcement has no legal targets
        // or it is subsequently countered or fails to resolve.
        match trigger_limit {
            Some(TriggerLimit::Turn) => {
                let used = TriggerUse {
                    source: source.reference,
                    ability: u16::try_from(index).expect("ability index"),
                    origin: ability_origin,
                    acquired_link_cohort,
                    controller: None,
                };
                if self.state.triggered_this_turn.contains(&used) {
                    return false;
                }
                self.state.triggered_this_turn.push(used);
            }
            Some(TriggerLimit::Incarnation) => {
                let used = IncarnationTriggerUse {
                    source: source.reference,
                    definition,
                    ability: u16::try_from(index).expect("ability index"),
                    origin: ability_origin,
                    acquired_link_cohort,
                };
                if !self.record_incarnation_trigger_use(used) {
                    return false;
                }
            }
            None => {}
        }
        let modal_x = match definition.triggers()[index].program.first() {
            Some(Instruction::ModalUpToStat { maximum, .. }) => Some(
                u32::try_from(
                    self.stat_value(*maximum, source.reference, source.controller)
                        .max(0),
                )
                .unwrap_or(u32::MAX),
            ),
            _ => None,
        };
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
            additional_triggers: if trigger_limit.is_some()
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
            acquired_link_cohort,
            self_granted_activation: None,
            target_controller_lki: None,
            target_types_lki: None,
            target_type_slots: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            target_allocations: None,
            announced_division: None,
            captured_artifact_copy: None,
            source_copy_recipe: source
                .copy_recipe
                .filter(|r| r.needs_source_snapshot())
                .map(Box::new),
            source_ability_origin: ability_origin,
            mana_copy_sources: None,
            mana_provenance: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: modal_x,
            loyalty_x: None,
            reflexive_target_limit: None,
            attachment_trigger: event,
            acquired_trigger_definition: acquired.map(|grant| grant.definition),
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
                    definition.triggers()[index].targets,
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
                } = condition.without_incarnation_limit()
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
            source_quantity_lki: self.source_quantity_facts(
                definition.triggers()[index].program,
                definition.triggers()[index].targets,
                source,
            ),
            sacrificed_toughness: None,
            sacrificed_mana_value: None,
            sacrificed_power: None,
            source_attachment: source.attached_to,

            controller: source.controller,
            targets: Vec::new(),
            mode: None,
        });
        if source_prevention::condition_refers_to_attachment(condition) {
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .refer_objects(source.attached_to);
        }
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
            if let Some(event) = &item.event_object
                && matches!(event.facts, EventObjectFacts::RevealedCostCreature { .. })
                && self.object_ref(event.object.card) == event.object
                && self.state.objects[event.object.card as usize].zone == Zone::Hand
            {
                self.remember_public_hand(event.object);
            }
        }
        self.register_stack_zone_replacements(&item);
        self.record_committed_crime(&item);
        self.committed_activation_observers(&item);
        self.state.stack.push(item);
        let Some((caster, card, _)) = cast else {
            return;
        };
        self.record_cast(caster, card, Some(self.state.stack.len() - 1));
    }

    /// CR 115.9c compares target identities independently of their target groups.
    fn only_distinct_target(targets: &[BoundTarget]) -> Option<&BoundTarget> {
        let (first, rest) = targets.split_first()?;
        rest.iter()
            .all(|bound| bound.target == first.target && bound.object == first.object)
            .then_some(first)
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
        spell_traits: SpellCastTraits,
    ) -> bool {
        match filter {
            CastFilter::SourceChosenCreatureType => {
                matches!(source.entry_choice, Some(EntryChoiceValue::CreatureType(typ)) if spell_traits.subtypes.has(card, typ.name()))
            }
            CastFilter::CreatureManaValueEqualsSourcePower => {
                card.is_creature() && i128::from(mana_value) == source.power
            }
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
                        spell_traits,
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
            CastFilter::FirstQualifiedCreature(qualifier) => {
                spell_traits.matches_qualifier(qualifier, card)
                    && self.state.players[caster.index()]
                        .turn_facts
                        .qualified_creature_cast_count(qualifier)
                        == 1
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
                        spell_traits,
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
                        spell_traits,
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
                    spell_traits,
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
                        spell_traits,
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
                        spell_traits,
                    )
            }
            CastFilter::TargetCount { filter, count } => {
                targets.len() == usize::from(count)
                    && self.cast_filter_matches_value(
                        spell_owner,
                        *filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                        spell_traits,
                    )
            }
            CastFilter::Targeted {
                filter: inner,
                target,
            }
            | CastFilter::OnlyTargeted {
                filter: inner,
                target,
            } => {
                if matches!(filter, CastFilter::OnlyTargeted { .. })
                    && Self::only_distinct_target(targets).is_none()
                {
                    return false;
                }
                self.cast_filter_matches_value(
                    spell_owner,
                    *inner,
                    card,
                    mana_value,
                    caster,
                    source,
                    targets,
                    spell_traits,
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
            _ => filter.matches_mana_value_with(card, card.fixed_mana_value(), |subtype| {
                spell_traits.subtypes.has(card, subtype)
            }),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Semantic cast references use the same exact event and observer context as qualification"
    )]
    pub(super) fn cast_semantic_targets(
        &self,
        owner: Option<Player>,
        filter: CastFilter,
        card: Card,
        mana_value: u64,
        caster: Player,
        source: &ObjectView,
        targets: &[BoundTarget],
        spell_traits: SpellCastTraits,
    ) -> Vec<ObjectRef> {
        if !self.cast_filter_matches_value(
            owner,
            filter,
            card,
            mana_value,
            caster,
            source,
            targets,
            spell_traits,
        ) {
            return Vec::new();
        }
        let mut objects = Vec::new();
        match filter {
            CastFilter::Targeted { filter: inner, .. }
            | CastFilter::OnlyTargeted { filter: inner, .. } => {
                objects.extend(self.cast_semantic_targets(
                    owner,
                    *inner,
                    card,
                    mana_value,
                    caster,
                    source,
                    targets,
                    spell_traits,
                ));
                for bound in targets {
                    if self.cast_filter_matches_value(
                        owner,
                        filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        std::slice::from_ref(bound),
                        spell_traits,
                    ) && let Some(object) = bound.object
                    {
                        objects.push(object);
                    }
                }
            }
            CastFilter::CasterOffTurn { filter }
            | CastFilter::TargetCount { filter, .. }
            | CastFilter::Colors { filter, .. }
            | CastFilter::ManaSymbols { filter, .. }
            | CastFilter::ManaValueMaximum { filter, .. }
            | CastFilter::ManaValueMinimum { filter, .. } => {
                objects.extend(self.cast_semantic_targets(
                    owner,
                    *filter,
                    card,
                    mana_value,
                    caster,
                    source,
                    targets,
                    spell_traits,
                ));
            }
            CastFilter::AnyOf(filters) => {
                for &filter in filters {
                    objects.extend(self.cast_semantic_targets(
                        owner,
                        filter,
                        card,
                        mana_value,
                        caster,
                        source,
                        targets,
                        spell_traits,
                    ));
                }
            }
            _ => {}
        }
        objects.sort_unstable();
        objects.dedup();
        objects
    }

    pub(super) fn land_play_triggers(
        &mut self,
        player: Player,
        land: ObjectRef,
        zone: Zone,
        owner: Player,
    ) {
        let origin = (zone, owner == player);
        self.state.players[player.index()]
            .turn_facts
            .played_from_outside_own_hand |= origin != (Zone::Hand, true);
        for (source, grant) in self.indexed_trigger_sources(&self.state.cast_sources) {
            if source.controller != player {
                continue;
            }
            for (index, ability) in source.trigger_templates_from(grant.as_ref()) {
                if ability.condition.event() == TriggerCondition::SpellCastOrLandPlayed
                    && ability.condition.matches_play_origin(Some(origin))
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects([land]);
                }
            }
        }
    }

    fn cast_matches_entry_number(&self, source: &ObjectView, event: &CastPaymentEvent) -> bool {
        let Some(EntryChoiceValue::Number(number)) = source.entry_choice else {
            return false;
        };
        if event.mana_value == u64::from(number) {
            return true;
        }
        if !event.card.is_creature() {
            return false;
        }
        let stats = self.stats(event.object.card);
        let number = i128::from(number);
        stats.power == number || stats.toughness == number
    }

    pub(super) fn record_cast(&mut self, caster: Player, card: Card, stack: Option<usize>) {
        // Every committed path classifies the selected spell face, including
        // manual direct-payment insertion. The physical front may differ.
        let card = stack
            .and_then(|i| self.state.stack[i].source.map(|source| source.card))
            .unwrap_or(card);
        if let Some(index) = stack {
            if let Some(subtype) = card.cast_controlled_subtype() {
                let controlled = self.state.objects.iter().enumerate().any(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == caster
                        && self.has_subtype(id as CardId, subtype)
                });
                self.state.stack[index]
                    .cast
                    .as_mut()
                    .expect("committed spell has a cast record")
                    .controlled_subtype_at_cast = controlled;
            }
            let item = self.state.stack[index].clone();
            self.state.void_active_this_turn |= item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Warp);
            self.expire_land_mana_on_exile_cast(&item);
            self.commit_linked_exile_cast(&item);
            self.commit_linked_free_cast(&item);
            self.commit_exile_zero_cast(item.id);
            self.commit_library_entry_counter(&item);
        }
        self.commit_suspend_cast(stack);
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
        let origin = stack
            .and_then(|i| self.state.stack[i].cast)
            .map(|cast| (cast.origin, spell_owner == Some(caster)));
        let spell_subtypes = stack
            .and_then(|i| self.state.stack[i].source)
            .map_or_else(GrantedSubtypes::default, |source| {
                self.granted_subtypes(source.id)
            });
        let spell_traits = SpellCastTraits {
            subtypes: spell_subtypes,
            keywords: if card.is_creature() {
                stack.map_or_else(
                    || card.keywords(),
                    |i| {
                        let item = &self.state.stack[i];
                        let source = item.source.expect("committed creature spell");
                        if self.state.objects[source.id as usize].card == card {
                            self.keywords(source.id)
                        } else {
                            item.source_keywords
                        }
                    },
                )
            } else {
                card.keywords()
            },
        };
        let mana_value = stack.map_or_else(
            || card.fixed_mana_value(),
            |i| self.state.stack[i].mana_value(),
        );
        let player = &mut self.state.players[caster.index()];
        let cast_kinds = card.kind_mask();
        let first_kinds = cast_kinds & !player.turn_facts.cast_card_kinds;
        player.turn_facts.cast_card_kinds |= cast_kinds;
        player.turn_facts.played_from_outside_own_hand |=
            origin.is_some_and(|origin| origin != (Zone::Hand, true));
        player.turn_facts.cast_spell_from_own_hand |= origin == Some((Zone::Hand, true));
        player.turn_facts.cast_mana_value_at_least_four |= mana_value >= 4;
        let count = &mut player.spells_cast_this_turn;
        *count = count.checked_add(1).capacity("spell count exhausted");
        let ordinal = *count;
        if CastFilter::InstantSorcery.matches(card) {
            let count = &mut player.instant_sorcery_cast_this_turn;
            *count = count
                .checked_add(1)
                .capacity("qualifying spell count exhausted");
        }
        if CastFilter::Creature.matches(card) {
            player
                .turn_facts
                .record_qualified_creature_cast(card, spell_traits);
            let count = &mut player.creature_spells_cast_this_turn;
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
                    cave_mana_spent: item.cave_mana_spent(),
                    web_slinging_return_mana_value: cast.web_slinging_return_mana_value,
                })
        });
        let indexed = self.indexed_trigger_sources(&self.state.cast_sources);
        let controlled = self.controlled_cast_trigger_sources(caster);
        let sources = indexed
            .into_iter()
            .map(|(source, grant)| (source, grant, None))
            .chain(
                controlled
                    .into_iter()
                    .map(|(source, grant, ability)| (source, Some(grant), Some(ability))),
            );
        for (source, grant, exact_ability) in sources {
            for (index, ability) in source.trigger_templates_from(grant.as_ref()) {
                if exact_ability.is_some_and(|exact| index != exact) {
                    continue;
                }
                let condition = ability.condition.event();
                let mode_count = if condition == TriggerCondition::ModalSpellCastByYou {
                    let Some(count) = stack
                        .and_then(|i| self.state.stack[i].mode)
                        .and_then(|mode| card.selected_mode_count(mode))
                    else {
                        continue;
                    };
                    Some(u32::from(count))
                } else {
                    None
                };
                let event = match condition {
                    TriggerCondition::FirstSpellKindOrOtherCreatureType { kinds, subtype } => {
                        if first_kinds & kinds == 0
                            && !self.is_first_other_creature_type_cast(
                                caster,
                                source.reference,
                                subtype,
                                card,
                                spell_subtypes,
                            )
                        {
                            continue;
                        }
                        TriggerCondition::SpellCast {
                            relation: CastRelation::You,
                            filter: CastFilter::Any,
                            ordinal: None,
                        }
                    }
                    TriggerCondition::SpellCastUsingSourceMana(filter) => {
                        TriggerCondition::SpellCast {
                            relation: CastRelation::You,
                            filter,
                            ordinal: None,
                        }
                    }
                    TriggerCondition::SpellCastOrLandPlayed
                    | TriggerCondition::ModalSpellCastByYou => TriggerCondition::SpellCast {
                        relation: CastRelation::You,
                        filter: CastFilter::Any,
                        ordinal: None,
                    },
                    event => event,
                };
                if let TriggerCondition::SpellCast {
                    relation,
                    filter,
                    ordinal: wanted,
                } = event
                {
                    let related = match relation {
                        CastRelation::You => source.controller == caster,
                        CastRelation::Opponent => source.controller != caster,
                        CastRelation::Any => true,
                    };
                    if related
                        && (!matches!(
                            ability.condition.event(),
                            TriggerCondition::SpellCastUsingSourceMana(_)
                        ) || stack.is_some_and(|index| {
                            self.state.stack[index].paid_mana_from(source.reference)
                        }))
                        && ability.condition.matches_play_origin(origin)
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
                        } else if filter == CastFilter::SourceChosenNumberMatchesSpell {
                            cast_event
                                .as_ref()
                                .is_some_and(|event| self.cast_matches_entry_number(&source, event))
                        } else if filter == CastFilter::MulticoloredInstantSorcery {
                            CastFilter::InstantSorcery.matches(card)
                                && stack
                                    .and_then(|i| self.state.stack[i].source)
                                    .map_or(card.colors(), |source| source.facts.colors)
                                    .count_ones()
                                    >= 2
                        } else if filter == CastFilter::Colorless {
                            stack
                                .and_then(|i| self.state.stack[i].source)
                                .map_or(card.colors() == 0, |source| source.facts.colors == 0)
                        } else {
                            self.cast_filter_matches_value(
                                spell_owner,
                                filter,
                                card,
                                mana_value,
                                caster,
                                &source,
                                stack
                                    .map(|i| self.state.stack[i].targets.as_slice())
                                    .unwrap_or(&[]),
                                spell_traits,
                            )
                        })
                        && wanted.is_none_or(|wanted| wanted == ordinal)
                        && self.queue_trigger_from(&source, index, grant.map(Box::new))
                    {
                        if let Some(event) = cast_event.as_ref() {
                            let targets = stack
                                .map(|i| self.state.stack[i].targets.as_slice())
                                .unwrap_or(&[]);
                            let objects = self.cast_semantic_targets(
                                spell_owner,
                                filter,
                                card,
                                event.mana_value,
                                caster,
                                &source,
                                targets,
                                spell_traits,
                            );
                            let packet = self.state.pending_triggers.last_mut().unwrap();
                            packet.refer_objects(std::iter::once(event.object).chain(objects));
                        }
                        if ability
                            .program
                            .contains(&Instruction::CopyTargetsFromCastSpell)
                            && let Some(event) = cast_event.as_ref()
                        {
                            let recipe = self
                                .copy_recipe(event.object)
                                .unwrap_or_else(|| CopyRecipe::new(event.card));
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .captured_artifact_copy = Some(Box::new(recipe));
                        }
                        // A cast trigger owns the triggering spell's chosen X even after that
                        // spell is countered or leaves the stack. ChosenX reads this packet.
                        let chosen_x =
                            mode_count.or_else(|| stack.and_then(|i| self.state.stack[i].x));
                        self.state.pending_triggers.last_mut().unwrap().x = chosen_x;
                        if ability.condition.mana_spent_rule().is_some()
                            || ability.condition.requires_treasure_mana()
                            || cast_symbol_quantities::selected(ability.program)
                            || ability.program.iter().any(|instruction| {
                                matches!(
                                    instruction,
                                    Instruction::ChangeCastSpellTargets
                                        | Instruction::MarkCastDreamExile
                                        | Instruction::FlipCoinForCapturedSpell { .. }
                                        | Instruction::GainLifeScaled(_, AmountValue::SpellColors)
                                        | Instruction::DamageScaled(
                                            AmountValue::CastSpellManaValue
                                        )
                                        | Instruction::DamageRememberPlayersScaled(
                                            AmountValue::CastSpellManaValue
                                        )
                                        | Instruction::CreateTokensWithTemporaryKeywords { .. }
                                        | Instruction::DamagePlayers(PlayerSet::CastingPlayer, _)
                                        | Instruction::DamagePlayersScaled(PlayerSet::CastingPlayer, _)
                                        | Instruction::LoseLife(PlayerSet::CastingPlayer, _)
                                        | Instruction::ProhibitAttacksOnControllerOrPlaneswalkersThisTurn(
                                            PlayerSet::CastingPlayer
                                        )
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
        if let Some(event) = cast_event.as_ref() {
            self.record_creature_type_cast(event, cast_kinds, spell_subtypes);
        }
        // Source-cast abilities live on the actual stack spell, outside the
        // battlefield observer index. Capture copies below before any priority.
        if let Some(index) = stack {
            let item = &self.state.stack[index];
            if !item.is_copy {
                let source = self.object_view(item.source.expect("cast spell has source").id);
                for (ability, trigger) in source.trigger_templates() {
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
        self.queue_cast_turn_callbacks(
            caster,
            cast_event.as_ref().map(|event| event.object),
            card.kind_mask() & 1 == 0,
        );
        self.temporary_cast_triggers(caster, card, stack, spell_traits);
        self.capture_cast_creature_subjects(stack, first_trigger);
        self.capture_cast_callbacks(stack, first_trigger, cast_event, spell_subtypes);
        if let Some(index) = stack {
            self.capture_mana_copies(index);
        }
    }

    /// Apply one simultaneous tap event, then collect source-owned triggers.
    /// Entry status and virtual mana funding never use this transition.
    pub(super) fn tap_objects_as(&mut self, actor: Player, ids: &[CardId]) {
        // Keep the ordinary single-source mana tap allocation-free.
        if let &[id] = ids {
            if self.tap_permanent(id) {
                self.finish_tap_event(&[(actor, id)]);
            }
        } else {
            self.tap_objects_by(ids.iter().map(|&id| (actor, id)));
        }
    }

    /// Setup taps in fixture tests use the active player.
    #[cfg(feature = "test-support")]
    pub(super) fn tap_objects(&mut self, ids: &[CardId]) {
        self.tap_objects_as(self.state.active, ids);
    }

    /// Each player can be instructed to tap their own chosen part of the event.
    pub(super) fn tap_objects_by(
        &mut self,
        assignments: impl IntoIterator<Item = (Player, CardId)>,
    ) -> usize {
        let changed: Vec<_> = assignments
            .into_iter()
            .filter(|&(_, id)| self.tap_permanent(id))
            .collect();
        self.finish_tap_event(&changed);
        changed.len()
    }

    fn tap_permanent(&mut self, id: CardId) -> bool {
        let object = &mut self.state.objects[id as usize];
        if object.zone != Zone::Battlefield || object.tapped {
            return false;
        }
        object.tapped = true;
        true
    }

    fn finish_tap_event(&mut self, changed: &[(Player, CardId)]) {
        // Every observer sees the complete simultaneous tapped result. No queued
        // effect resolves until the surrounding action or instruction completes.
        for &(_, id) in changed {
            self.record_tap(self.object_ref(id));
        }
        for &(_, id) in changed {
            self.tapped_triggers(id);
        }
        self.tapped_batch_triggers(changed);
    }

    fn tapped_batch_triggers(&mut self, changed: &[(Player, CardId)]) {
        if changed.is_empty() || self.state.tap_sources.is_empty() {
            return;
        }
        self.attached_tap_triggers(changed);
        let sources: Vec<_> = self
            .state
            .tap_sources
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.trigger_templates() {
                match ability.condition.event() {
                    condition @ (TriggerCondition::ControlledCreaturesBecomeTapped
                    | TriggerCondition::ControlledPermanentsOfSubtypeTapped {
                        ..
                    }) => {
                        let matching: Vec<_> = changed
                            .iter()
                            .filter(|&&(_, id)| {
                                let object = &self.state.objects[id as usize];
                                object.controller == source.controller
                                    && if let TriggerCondition::ControlledPermanentsOfSubtypeTapped {
                                        subtype,
                                        nontoken,
                                    } = condition
                                    {
                                        (!nontoken || !object.is_token())
                                            && self.has_subtype(id, subtype)
                                    } else {
                                        self.is_creature(id)
                                    }
                            })
                            .map(|&(_, id)| self.object_ref(id))
                            .collect();
                        self.queue_tapped_objects(&source, index, matching);
                    }
                    TriggerCondition::ControlledCreatureFirstTapOwnTurn
                        if self.state.active == source.controller =>
                    {
                        for &(_, id) in changed {
                            let object = &self.state.objects[id as usize];
                            let reference = self.object_ref(id);
                            if object.controller == source.controller
                                && self.is_creature(id)
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
                    TriggerCondition::OpponentCreatureTappedByYou { one_or_more } => {
                        let matching: Vec<_> = changed
                            .iter()
                            .filter(|&&(actor, id)| {
                                actor == source.controller
                                    && self.state.objects[id as usize].controller != actor
                                    && self.is_creature(id)
                            })
                            .map(|&(_, id)| self.object_ref(id))
                            .collect();
                        if one_or_more {
                            self.queue_tapped_objects(&source, index, matching);
                        } else {
                            for reference in matching {
                                if self.queue_trigger(&source, index) {
                                    self.state
                                        .pending_triggers
                                        .last_mut()
                                        .unwrap()
                                        .refer_objects([reference]);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn queue_tapped_objects(
        &mut self,
        source: &ObjectView,
        index: usize,
        matching: Vec<ObjectRef>,
    ) {
        if !matching.is_empty() && self.queue_trigger(source, index) {
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .refer_objects(matching);
        }
    }

    fn tapped_triggers(&mut self, id: CardId) {
        if !self.has_trigger(id, |ability| {
            matches!(
                ability.condition.event(),
                TriggerCondition::SelfBecomesTapped | TriggerCondition::SelfEntersOrBecomesTapped
            )
        }) {
            return;
        }
        let source = self.object_view(id);
        for (index, ability) in source.trigger_templates() {
            if matches!(
                ability.condition.event(),
                TriggerCondition::SelfBecomesTapped | TriggerCondition::SelfEntersOrBecomesTapped
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
        self.defender_attack_group_triggers(actor, declared);
        self.crewed_vehicle_attack_triggers(declared);
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
                            action: false,
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
                        action: false,
                        additional_objects: rest.to_vec(),
                    }));
                    self.state.pending_triggers.push(packet);
                }
            }
        }
        if !declared.is_empty() {
            self.queue_turn_callbacks(crate::cards::TurnCallbackEvent::Attack, actor, None);
            let packets: Vec<_> = self
                .state
                .delayed_attack_this_turn
                .as_deref()
                .into_iter()
                .flatten()
                .filter_map(|item| {
                    let StackKind::TriggeredAbility { ability } = item.kind else {
                        return None;
                    };
                    let event = item.trigger_definition().triggers()[usize::from(ability)]
                        .condition
                        .event();
                    (item.controller == actor
                        || matches!(event, TriggerCondition::DelayedCreatureAttacksSubtypeAny(_)))
                    .then(|| (item.clone(), event))
                })
                .collect();
            for (mut packet, event) in packets {
                if let TriggerCondition::DelayedCreatureAttacksSubtypeAny(subtypes) = event {
                    for attack in declared {
                        if subtypes
                            .iter()
                            .any(|name| self.has_subtype(attack.card, name))
                        {
                            let mut trigger = packet.clone();
                            trigger.id = self.next_stack_id();
                            trigger.entering_object = Some(self.object_ref(attack.card));
                            self.state.pending_triggers.push(trigger);
                        }
                    }
                    continue;
                }
                if event == TriggerCondition::DelayedCreatureAttacksAlone {
                    let Some(attacker) = alone else { continue };
                    packet.entering_object = Some(attacker);
                }
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
                let object = &self.state.objects[id as usize];
                match object.zone {
                    Zone::Battlefield => object.controller == actor,
                    Zone::Graveyard => object.owner == actor && object.is_card(),
                    _ => false,
                }
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.trigger_templates() {
                if source.zone == Zone::Graveyard
                    && ability.condition.origin() != Some(TargetZone::Graveyard)
                {
                    continue;
                }
                if let TriggerCondition::FirstAttackWithTotalPowerAtLeast(minimum) =
                    ability.condition.event()
                    && power_event
                        .previous_maximum
                        .is_none_or(|prior| prior < i128::from(minimum))
                    && power_event.total >= i128::from(minimum)
                    && self.queue_attack_trigger(&source, index)
                {
                    let objects: Vec<_> =
                        declared.iter().map(|a| self.object_ref(a.card)).collect();
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects(objects);
                }
                if ability.condition.event() == TriggerCondition::ControlledCreatureAttacksAlone
                    && let Some(attacker) = alone
                    && self.queue_attack_trigger(&source, index)
                {
                    // This existing field holds a generic exact event object.
                    // Capture the declared attacker, not a target or later combat count.
                    let counters = ability
                        .program
                        .iter()
                        .any(|instruction| {
                            matches!(
                                instruction,
                                Instruction::SurveilScaled(AmountValue::EventObjectTotalCounters)
                            )
                        })
                        .then(|| self.counter_amounts(attacker.card).total());
                    let item = self.state.pending_triggers.last_mut().unwrap();
                    item.entering_object = Some(attacker);
                    if let Some(counters) = counters {
                        item.set_event_object(EventObjectView {
                            object: attacker,
                            facts: EventObjectFacts::CounterTotal(counters),
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                    }
                }
                let condition = ability.condition.event();
                if condition == TriggerCondition::ControlledSuspectedCreaturesAttack {
                    let attackers: Vec<_> = declared
                        .iter()
                        .filter(|attack| self.is_suspected(attack.card))
                        .map(|attack| self.object_ref(attack.card))
                        .collect();
                    if !attackers.is_empty() && self.queue_attack_trigger(&source, index) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .refer_objects(attackers);
                    }
                }
                enum AttackRequirement {
                    Any,
                    Keyword(Keywords),
                    Counter(CounterKind),
                    Subtype {
                        subtype: &'static str,
                        exclude: bool,
                    },
                    AnySubtype(&'static [&'static str]),
                }
                let each_attacker = match condition {
                    TriggerCondition::ControlledCreatureAttacks
                    | TriggerCondition::ControlledCreatureAttacksOrBlocks => {
                        Some(AttackRequirement::Any)
                    }
                    TriggerCondition::ControlledCreatureAttacksWithKeyword(keyword) => {
                        Some(AttackRequirement::Keyword(keyword))
                    }
                    TriggerCondition::ControlledCreatureAttacksWithCounter(kind) => {
                        Some(AttackRequirement::Counter(kind))
                    }
                    TriggerCondition::ControlledSubtypeAttacks {
                        subtype,
                        exclude,
                        one_or_more: false,
                    } => Some(AttackRequirement::Subtype { subtype, exclude }),
                    TriggerCondition::ControlledSubtypeAttacksAny(subtypes) => {
                        Some(AttackRequirement::AnySubtype(subtypes))
                    }
                    _ => None,
                };
                if let Some(requirement) = each_attacker {
                    for attacker in declared {
                        let qualifies = match requirement {
                            AttackRequirement::Any => true,
                            AttackRequirement::Keyword(keyword) => {
                                keyword == Keywords::NONE
                                    || self.keywords(attacker.card).contains(keyword)
                            }
                            AttackRequirement::Counter(kind) => {
                                self.counter_count(attacker.card, kind) > 0
                            }
                            AttackRequirement::Subtype { subtype, exclude } => {
                                self.has_subtype(attacker.card, subtype) != exclude
                            }
                            AttackRequirement::AnySubtype(subtypes) => subtypes
                                .iter()
                                .any(|subtype| self.has_subtype(attacker.card, subtype)),
                        };
                        if !qualifies {
                            continue;
                        }
                        let object = self.object_ref(attacker.card);
                        let facts =
                            EventObjectFacts::DamageActor(self.damage_source(attacker.card));
                        if self.queue_attack_trigger(&source, index) {
                            let trigger = self.state.pending_triggers.last_mut().unwrap();
                            if matches!(
                                condition,
                                TriggerCondition::ControlledSubtypeAttacksAny(_)
                                    | TriggerCondition::ControlledCreatureAttacksWithCounter(_)
                            ) {
                                trigger.entering_object = Some(object);
                            }
                            trigger.set_event_object(EventObjectView {
                                object,
                                facts,
                                semantic_objects: None,
                                conniver_creature: None,
                                subtype_lki: None,
                            });
                        }
                    }
                }
                if ability.condition.event()
                    == TriggerCondition::ControlledCreatureAttacksPlayerAlone
                    && let Some(attacker) = alone
                    && matches!(declared[0].defender, Some(CombatDefender::Player(_)))
                    && self.queue_attack_trigger(&source, index)
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
                            && self.queue_attack_trigger(&source, index)
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
                    for attacker in declared {
                        if self.state.objects[attacker.card as usize].controller == actor
                            && ability_world::CharacteristicWorld::colors(self, attacker.card)
                                & colors
                                != 0
                            && self.queue_attack_trigger(&source, index)
                        {
                            let object = self.object_ref(attacker.card);
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .refer_objects([object]);
                        }
                    }
                }
                if matches!(
                    condition,
                    TriggerCondition::AttackPlayer(_)
                        | TriggerCondition::AttackPlayerExcludingSubtype(_)
                        | TriggerCondition::AttackPlayerWithPowerAtLeast(_)
                ) {
                    // Qualification belongs to this declaration, not all live attackers.
                    // A creature put onto the battlefield attacking did not attack.
                    for player in [Player::P0, Player::P1] {
                        let matching: Vec<_> = declared
                            .iter()
                            .filter(|attack| {
                                attack.defender == Some(CombatDefender::Player(player))
                                    && match condition {
                                        TriggerCondition::AttackPlayer(filter) => self
                                            .creature_matches(
                                                filter,
                                                actor,
                                                source.reference,
                                                attack.card,
                                            ),
                                        TriggerCondition::AttackPlayerExcludingSubtype(subtype) => {
                                            !self.has_subtype(attack.card, subtype)
                                        }
                                        TriggerCondition::AttackPlayerWithPowerAtLeast(power) => {
                                            self.object_view(attack.card).power >= i128::from(power)
                                        }
                                        _ => unreachable!(),
                                    }
                            })
                            .map(|attack| self.object_ref(attack.card))
                            .collect();
                        if !matching.is_empty() && self.queue_attack_trigger(&source, index) {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .refer_objects(matching);
                        }
                    }
                }
                if let TriggerCondition::ControlledCreatureWithKeywordAttackDeclaration(keyword) =
                    condition
                {
                    let matching: Vec<_> = declared
                        .iter()
                        .filter(|attack| self.keywords(attack.card).contains(keyword))
                        .map(|attack| self.object_ref(attack.card))
                        .collect();
                    if !matching.is_empty() && self.queue_attack_trigger(&source, index) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .refer_objects(matching);
                    }
                }
                let subtype_attack = match ability.condition.event() {
                    TriggerCondition::ControlledSubtypeAttacks {
                        subtype,
                        exclude,
                        one_or_more: true,
                    } => Some((subtype, exclude, 1)),
                    TriggerCondition::ControlledSubtypeAttackDeclaration(subtype) => {
                        Some((subtype, false, 1))
                    }
                    TriggerCondition::ControlledSubtypeAttackAtLeast { subtype, minimum } => {
                        Some((subtype, false, usize::from(minimum)))
                    }
                    _ => None,
                };
                if let Some((subtype, exclude, minimum)) = subtype_attack {
                    let matching: Vec<_> = declared
                        .iter()
                        .filter(|attack| self.has_subtype(attack.card, subtype) != exclude)
                        .map(|attack| self.object_ref(attack.card))
                        .collect();
                    if matching.len() >= minimum
                        && let Some((&object, rest)) = matching.split_first()
                        && self.queue_attack_trigger(&source, index)
                    {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                            object,
                            action: false,
                            additional_objects: rest.to_vec(),
                        }));
                    }
                }
                if let TriggerCondition::AttackDeclared {
                    minimum,
                    power_min,
                    guard,
                }
                | TriggerCondition::CreaturesAttackDeclared {
                    minimum,
                    power_min,
                    guard,
                } = ability.condition.event()
                {
                    let explicit_creatures = matches!(
                        ability.condition.event(),
                        TriggerCondition::CreaturesAttackDeclared { .. }
                    );
                    let mut count = 0usize;
                    let mut objects = Vec::new();
                    for attack in declared {
                        let object = &self.state.objects[attack.card as usize];
                        if object.controller == actor
                            && object.zone == Zone::Battlefield
                            && self.is_creature(attack.card)
                            && power_min.is_none_or(|power| {
                                self.object_view(attack.card).power >= i128::from(power)
                            })
                        {
                            count += 1;
                            if explicit_creatures {
                                objects.push(self.object_ref(attack.card));
                            }
                        }
                    }
                    // This is an event qualification, never an intervening-if recheck.
                    // Player-only clauses do not allocate or retain a creature roster.
                    if count >= usize::from(minimum)
                        && self.step_guard(guard, actor, source.reference, source.tapped)
                        && self.queue_attack_trigger(&source, index)
                    {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .refer_objects(objects);
                    }
                }
            }
        }
    }

    /// One event per defender player in this declaration, with exact non-targeted subjects.
    fn defender_attack_group_triggers(&mut self, actor: Player, declared: &[AttackDeclaration]) {
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.has_trigger(id, |ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::CreaturesAttackYou
                                | TriggerCondition::CreaturesAttackYouOrPlaneswalkersAtLeast(_)
                                | TriggerCondition::PlayerAttacksWithAtLeast(_)
                                | TriggerCondition::OpponentAttacked
                        )
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
            let opponent_attacked = declared
                .iter()
                .any(|row| row.defender == Some(CombatDefender::Player(source.controller.other())));
            for (index, ability) in source.trigger_templates() {
                match ability.condition.event() {
                    TriggerCondition::OpponentAttacked if opponent_attacked => {
                        self.queue_attack_trigger(&source, index);
                    }
                    TriggerCondition::PlayerAttacksWithAtLeast(minimum)
                        if declared.len() >= usize::from(minimum)
                            && self.queue_attack_trigger(&source, index) =>
                    {
                        let cohort: Vec<_> = declared
                            .iter()
                            .map(|row| self.object_ref(row.card))
                            .collect();
                        let packet = self.state.pending_triggers.last_mut().unwrap();
                        packet.event_player = Some(actor);
                        packet.refer_objects(cohort);
                    }
                    TriggerCondition::CreaturesAttackYouOrPlaneswalkersAtLeast(minimum)
                        if source.controller != actor
                            && declared
                                .iter()
                                .filter(|attack| {
                                    self.attack_threatens_player_or_planeswalker(
                                        attack,
                                        source.controller,
                                    )
                                })
                                .take(usize::from(minimum))
                                .count()
                                >= usize::from(minimum)
                            && self.queue_attack_trigger(&source, index) =>
                    {
                        let cohort: Vec<_> = declared
                            .iter()
                            .map(|row| self.object_ref(row.card))
                            .collect();
                        let packet = self.state.pending_triggers.last_mut().unwrap();
                        packet.event_player = Some(actor);
                        packet.refer_objects(cohort);
                    }
                    TriggerCondition::CreaturesAttackYou => {
                        if let Some((&first, rest)) = attackers.split_first()
                            && self.queue_attack_trigger(&source, index)
                        {
                            self.state
                                .pending_triggers
                                .last_mut()
                                .unwrap()
                                .reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                                object: first,
                                action: false,
                                additional_objects: rest.to_vec(),
                            }));
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub(super) fn self_block_triggers(&mut self, declared: &[CardId], first_new_block: usize) {
        let observers: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_view(id))
            .collect();
        for source in observers {
            for (index, ability) in source.trigger_templates() {
                if ability.condition.event() != TriggerCondition::ControlledCreatureAttacksOrBlocks
                {
                    continue;
                }
                for (position, &id) in declared.iter().enumerate() {
                    if !declared[..position].contains(&id)
                        && self.on_battlefield(id)
                        && self.state.objects[id as usize].controller == source.controller
                        && self.queue_trigger(&source, index)
                    {
                        let object = self.object_ref(id);
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .entering_object = Some(object);
                    }
                }
            }
        }
        for (index, &id) in declared.iter().enumerate() {
            if declared[..index].contains(&id)
                || !self.has_trigger(id, |ability| {
                    matches!(
                        ability.condition.event(),
                        TriggerCondition::SelfAttacksOrBlocks
                            | TriggerCondition::SelfBlocksCreatureWithKeyword(_)
                    )
                })
            {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.trigger_templates() {
                match ability.condition.event() {
                    TriggerCondition::SelfAttacksOrBlocks => {
                        self.queue_trigger(&source, index);
                    }
                    TriggerCondition::SelfBlocksCreatureWithKeyword(keyword) => {
                        for pair in first_new_block..self.state.blocks.len() {
                            let (blocker, attacker) = self.state.blocks[pair];
                            if blocker == id
                                && self.keywords(attacker).contains(keyword)
                                && self.queue_trigger(&source, index)
                            {
                                let object = self.object_ref(attacker);
                                self.state
                                    .pending_triggers
                                    .last_mut()
                                    .unwrap()
                                    .refer_objects([object]);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    /// A creature becoming blocked and a pair of creatures meeting in combat
    /// are different events. Multiple blockers share only the former.
    pub(super) fn block_relationship_triggers(
        &mut self,
        first_new_block: usize,
        newly_blocked: &[CardId],
    ) {
        for pair in first_new_block..self.state.blocks.len() {
            let (blocker, attacker) = self.state.blocks[pair];
            for (id, other) in [(blocker, attacker), (attacker, blocker)] {
                self.block_subject_triggers(
                    id,
                    other,
                    TriggerCondition::SelfBlocksOrBecomesBlockedByCreature,
                );
            }
        }
        if newly_blocked.is_empty() {
            return;
        }
        for &id in newly_blocked {
            self.block_subject_triggers(id, id, TriggerCondition::SelfBecomesBlocked);
        }
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.has_trigger(id, |ability| {
                        matches!(
                            ability.condition.event(),
                            TriggerCondition::ControlledSubtypeBecomesBlocked(_)
                                | TriggerCondition::ControlledCreatureBecomesBlocked
                        )
                    })
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.trigger_templates() {
                let subtype = match ability.condition.event() {
                    TriggerCondition::ControlledSubtypeBecomesBlocked(subtype) => Some(subtype),
                    TriggerCondition::ControlledCreatureBecomesBlocked => None,
                    _ => continue,
                };
                for &id in newly_blocked {
                    if self.state.objects[id as usize].controller == source.controller
                        && self.is_creature(id)
                        && subtype.is_none_or(|subtype| self.has_subtype(id, subtype))
                    {
                        self.queue_block_subject(&source, index, id);
                    }
                }
            }
        }
    }

    fn block_subject_triggers(&mut self, source: CardId, other: CardId, event: TriggerCondition) {
        let observes =
            |ability: &crate::cards::TriggeredAbility| ability.condition.event() == event;
        if !self.has_trigger(source, observes) {
            return;
        }
        let source = self.object_view(source);
        for (index, ability) in source.trigger_templates() {
            if observes(ability) {
                self.queue_block_subject(&source, index, other);
            }
        }
    }

    fn queue_block_subject(&mut self, source: &ObjectView, index: usize, id: CardId) {
        if self.queue_trigger(source, index) {
            let object = self.object_ref(id);
            let packet = self.state.pending_triggers.last_mut().unwrap();
            packet.entering_object = Some(object);
            packet.refer_objects([object]);
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
                for (index, ability) in source.trigger_templates() {
                    if ability.condition.event() == TriggerCondition::OpponentCreatureBlocksSmall
                        && self.queue_trigger(source, index)
                    {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .set_event_object(EventObjectView {
                                object,
                                facts: EventObjectFacts::Controller(actor),
                                semantic_objects: None,
                                conniver_creature: None,
                                subtype_lki: None,
                            });
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
            for (index, ability) in giver.trigger_templates() {
                let granted = match ability.condition.event() {
                    TriggerCondition::AttachedAttacksOrBlocks => false,
                    TriggerCondition::AttachedAttacks if !blocking => false,
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
                // In two-player combat the attacked player, planeswalker's
                // controller or battle's protector is the opponent of the attacker.
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
                if blocking {
                    self.queue_trigger_from(&source, index, Some(event));
                } else {
                    self.queue_attack_trigger_from(&source, index, Some(event));
                }
            }
        }
    }

    /// Attack-event multiplicity is separate from ordinary permanent trigger
    /// multipliers so shared ETB/attack conditions and delayed packets stay exact.
    fn queue_attack_trigger(&mut self, source: &ObjectView, index: usize) -> bool {
        self.queue_attack_trigger_from(source, index, None)
    }
    pub(super) fn queue_attack_trigger_from(
        &mut self,
        source: &ObjectView,
        index: usize,
        grant: Option<Box<AttachmentTriggerView>>,
    ) -> bool {
        if !self.queue_trigger_from(source, index, grant) {
            return false;
        }
        // A printed trigger limit caps all additional instances too.
        let queued = self.state.pending_triggers.last().unwrap();
        if queued
            .triggered_ability()
            .condition
            .trigger_limit()
            .is_some()
        {
            return true;
        }
        let extra = trigger_multiplicity::additional_attack_triggers(
            &self.characteristic_query(),
            source.id,
        );
        let item = self.state.pending_triggers.last_mut().unwrap();
        item.additional_triggers = item
            .additional_triggers
            .checked_add(extra)
            .capacity("attack trigger multiplicity exhausted");
        true
    }

    pub(super) fn attack_triggers_for_declaration(
        &mut self,
        source: &ObjectView,
        defender: Option<CombatDefender>,
        declared_count: usize,
        first_attack: bool,
    ) {
        let borrowed_decayed = source
            .acquired_triggers
            .as_ref()
            .is_some_and(|grant| grant.definition.keywords().contains(Keywords::DECAYED));
        if source.keywords.contains(Keywords::DECAYED) || borrowed_decayed {
            let printed_instances = if source.keywords.contains(Keywords::DECAYED) {
                self.characteristic_query().decayed_instances(source.id)
            } else {
                0
            };
            let instances = printed_instances + usize::from(borrowed_decayed);
            for _ in 0..instances {
                let mut item = self.activation_item(source.controller, source.id, 0);
                item.kind = StackKind::DecayedAttack;
                item.attachment_trigger = None;
                item.source_counter_lki = None;
                item.source_quantity_lki = None;
                item.additional_triggers = source
                    .additional_triggers
                    .checked_add(trigger_multiplicity::additional_attack_triggers(
                        &self.characteristic_query(),
                        source.id,
                    ))
                    .capacity("attack trigger multiplicity exhausted");
                self.state.pending_triggers.push(item);
            }
        }
        self.conditional_self_attack_triggers(source);
        let defending_player = match defender {
            Some(CombatDefender::Player(player)) => player,
            Some(CombatDefender::Planeswalker(object)) => {
                self.state.objects[object.card as usize].controller
            }
            Some(CombatDefender::Battle(object)) => self
                .battle_protector(object)
                .unwrap_or(self.state.active.other()),
            None => self.state.active.other(),
        };
        for (index, ability) in source.trigger_templates() {
            let condition = ability.condition.event();
            let matches = match condition {
                TriggerCondition::SelfAttacks
                | TriggerCondition::SelfAttacksDefendingPlayer
                | TriggerCondition::SelfAttacksIfEquipped
                | TriggerCondition::SelfAttacksOrBlocks
                | TriggerCondition::SelfEntersOrAttacks => true,
                TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype(subtype) => {
                    !self.controls_creature_subtype(defending_player, subtype)
                }
                TriggerCondition::SelfFirstAttack => first_attack,
                TriggerCondition::SelfAttacksPlayerWithMostLife => {
                    matches!(defender, Some(CombatDefender::Player(player))
                        if self.state.players[player.index()].life
                            >= self.state.players[player.other().index()].life)
                }
                TriggerCondition::SelfAttacksAlone => declared_count == 1,
                TriggerCondition::SelfAttacksWithOthers => declared_count > 1,
                TriggerCondition::SelfAttacksWhileSaddled
                | TriggerCondition::SelfEntersOrAttacksWhileSaddled => source.saddled,
                _ => false,
            };
            let event_player = matches!(
                condition,
                TriggerCondition::SelfAttacksDefendingPlayer
                    | TriggerCondition::SelfAttacksPlayerWithMostLife
                    | TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype(_)
            )
            .then_some(defending_player);
            if matches
                && self.queue_attack_trigger(source, index)
                && let Some(player) = event_player
            {
                self.state.pending_triggers.last_mut().unwrap().event_player = Some(player);
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
        // CR603.3b places ordinary triggers in APNAP order before abilities
        // that triggered from another ability triggering. Each pass retains
        // independent controller choices within that player's own batch.
        for observers in [false, true] {
            for player in [self.state.active, self.state.active.other()] {
                loop {
                    let mut indices = self
                        .state
                        .ordering_triggers
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| {
                            item.controller == player && item.observes_trigger() == observers
                        })
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
        }
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }

    pub(super) fn resolve_top(&mut self) {
        let mut item = self.state.stack.pop().unwrap();
        self.retain_departing_ability_for_triggers(&item);
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
        let original_single_spell_card_target = item.kind == StackKind::Spell
            && Self::only_distinct_target(&item.targets).is_some_and(|first| {
                matches!(first.target, Target::Card(_))
                    && first.object.is_some()
                    && item
                        .source
                        .is_some_and(|source| source.card.kind_mask() & (4 | 8) != 0)
            });
        let spec = self.target_spec(&item);
        // Preserve the whole original target set for relational legality. A departed
        // target still contributes its last controller, even though it cannot move.
        // A target defined by an attachment to an earlier target also depends
        // on that earlier target remaining legal. The empty case allocates nothing.
        let relation_illegal: Vec<_> = item
            .targets
            .iter()
            .copied()
            .filter(|bound| !self.resolution_target_relations_hold(&item, *bound, spec, true))
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
            | StackKind::DecayedAttack
            | StackKind::SuspendTick
            | StackKind::SuspendCast
            | StackKind::ReboundCast
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::ConspireCopy
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
                        let target = item.targets[0];
                        let mut movement = self.entry_move(source.id, Zone::Battlefield, false);
                        match target.target {
                            Target::Player(player) => movement.player_attachment = Some(player),
                            _ => movement.attachment = target.object,
                        }
                        movement.cast_event = item.actual_cast_event().map(Box::new);
                        movement.cast_payment =
                            item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                        movement.cast_origin =
                            item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
                        movement.gift = item.gift_context().map(Box::new);
                        movement.warp_controller = item.delayed_warp_controller();
                        movement.apply_impending_cost(item.cast);
                        self.stage_entries(vec![movement], None);
                        self.give_priority();
                        return;
                    }
                    Effect::Creature(..) | Effect::Permanent => {
                        let linked_cast_exile = if item.is_copy {
                            [None; 2]
                        } else if let Some(EventObjectFacts::LinkedExileCost(cards)) =
                            item.event_object.as_deref().map(|event| &event.facts)
                        {
                            *cards
                        } else {
                            [item.cast.and_then(|cast| cast.behold_exiled), None]
                        };
                        if source.card.other_face().is_some() {
                            self.state.objects[source.id as usize].card = source.card;
                        }
                        self.record_resolved_spell_name(&item);
                        if let Some((room, _)) = source.card.room_half() {
                            let before = self.chosen_spell_before(&item);
                            let mut movement =
                                entries::EntryMove::from_view(before, Zone::Battlefield, false);
                            movement.project_printed_face(room);
                            movement.before.name = source.card.name();
                            movement.before.colors = source.card.colors();
                            // If a replacement suspends this entry, its public stack
                            // object still has the selected half until the move.
                            self.state.objects[source.id as usize].card = source.card;
                            movement.cast_event = item.actual_cast_event().map(Box::new);
                            movement.cast_payment =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                            movement.cast_origin =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
                            movement.warp_controller = item.delayed_warp_controller();
                            if let Some(mut recipe) = self.copy_recipe(source.object) {
                                recipe.definition = room;
                                self.set_copy_recipe(source.object, recipe);
                                movement.before.copy_recipe = Some(recipe);
                            }
                            self.stage_entries(vec![movement], None);
                        } else if item.cast.is_some_and(|c| {
                            c.offspring == Some(true)
                                || c.kicker == Some(true)
                                || c.bargain == Some(true)
                                || c.evidence == Some(true)
                                || c.impending
                                || matches!(c.gift, Some(GiftDeclaration::Promised { .. }))
                                || matches!(
                                    c.method,
                                    CastMethod::Warp | CastMethod::Sneak | CastMethod::Disguise
                                )
                                || source.card.entry_counter_rules().iter().any(|r| {
                                    matches!(
                                        r.amount,
                                        EntryCounterAmount::ColorsSpent
                                            | EntryCounterAmount::ManaSpent
                                            | EntryCounterAmount::IfNotCastOrZero(_)
                                            | EntryCounterAmount::WebSlingingReturnedManaValue
                                    )
                                })
                        }) || item.x.is_some()
                            || self.has_cast_entry_counters(source.object)
                            || !source.card.graveyard_cast_entry_counters().is_empty()
                            || !source.card.hand_cast_entry_counters().is_empty()
                            || !source.card.cast_entry_counters().is_empty()
                            || !source.card.evoke_costs().is_empty()
                            || linked_cast_exile[0].is_some()
                            || source.card.entry_choice().is_some()
                            || source.card.entry_copy().is_some()
                            || source.card.entry_exile_counters().is_some()
                            || source.card.devour().is_some()
                            || self.has_entry_counter_replacements(source.card)
                            || self.may_enter_with_riot(source.card)
                            || source.card.kind_mask() & crate::cards::BATTLE_KIND != 0
                        {
                            let mut movement = self.entry_move(
                                source.id,
                                Zone::Battlefield,
                                item.cast.is_some_and(|c| c.method == CastMethod::Sneak),
                            );
                            movement.attacking = item.cast.and_then(|c| c.sneak_defender);
                            movement.sneak_paid =
                                item.cast.is_some_and(|c| c.method == CastMethod::Sneak);
                            movement.warp_controller = item.delayed_warp_controller();
                            movement.cast_event = item.actual_cast_event().map(Box::new);
                            movement.cast_payment =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                            movement.cast_origin =
                                item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
                            movement.evoked = item.cast.is_some_and(|c| c.evoked);
                            movement.apply_impending_cost(item.cast);
                            movement.linked_cast_exile = linked_cast_exile;
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
                            event.cast_origin =
                                item.cast.filter(|_| !item.is_copy).map(|cast| cast.origin);
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
                } | Instruction::DamageByTargetPower { .. }
                    | Instruction::LoseLifeScaled(_, AmountValue::TargetCreaturePower)
                    | Instruction::IfAmountAtLeast {
                        value: AmountValue::TargetCreaturePower,
                        ..
                    }
            )
        });
        let reads_target_attacking = program.iter().any(|instruction| {
            matches!(instruction, Instruction::DrawTargetControllerIfNotAttacking)
        });
        let reads_target_damage = program.iter().any(|instruction| {
            matches!(instruction, Instruction::DrawTargetControllerIfUndamaged(_))
        });
        let reads_target_legendary_kind = program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::DrawIfTargetLegendaryKind(_)));
        let reads_target_card = program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::EnsureGoblinArmyFor(PlayerSet::TargetController)
                    | Instruction::AmassGoblinsScaled {
                        player: PlayerSet::TargetController,
                        ..
                    }
                    | Instruction::DrawIfTargetController
                    | Instruction::Draw(PlayerSet::TargetController, _)
                    | Instruction::DrawScaled(PlayerSet::TargetController, _)
                    | Instruction::DrawPowerOfTwoChosenX(PlayerSet::TargetController)
                    | Instruction::DrawToHandSize {
                        players: PlayerSet::TargetController,
                        ..
                    }
                    | Instruction::CreateTokensFor {
                        players: PlayerSet::TargetController,
                        ..
                    }
                    | Instruction::DrawTargetControllerIfNotAttacking
                    | Instruction::ManifestDreadForTargetController
                    | Instruction::DrawIfTargetLegendaryKind(_)
                    | Instruction::LoseLifeTargetCardManaValue(_)
                    | Instruction::MayTargetController { .. }
                    | Instruction::SearchTargetController(_)
                    | Instruction::SearchTargetControllerWithEntryCounters { .. }
                    | Instruction::LoseLifeTargetController(_)
                    | Instruction::DamageTargetController(_)
                    | Instruction::DecreaseLeadingSpeed {
                        players: PlayerSet::TargetController,
                        ..
                    }
            )
        });
        let target_card_facts = (reads_target_power || reads_target_card || reads_target_damage)
            .then(|| {
                Box::new(
                    item.targets
                        .iter()
                        .filter_map(|bound| match bound.target {
                            Target::Card(card) => Some(TargetCardFacts {
                                object: self.object_ref(card),
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
                                damaged: reads_target_damage
                                    && self.state.objects[card as usize].damaged_this_turn,
                                attacking: reads_target_attacking
                                    && self.state.attackers.iter().any(|a| a.card == card),
                            }),
                            _ => None,
                        })
                        .collect(),
                )
            });
        self.state.resolving = Some(Resolution {
            power_difference: None,
            chosen_color: None,
            chosen_card_name: None,
            chosen_creature_type: None,
            target_counter_removal: None,
            linked_copy: None,
            blight_copy: None,
            cascade: None,
            target_card_facts,
            sacrificed_power: None,
            sacrificed_mana_value: None,
            exile_continuation: None,
            repeated_instruction_remaining: None,
            explore: None,
            manifest_sequence: None,
            evidence: None,
            connive: None,
            proliferate: None,
            item,
            program,
            pc: 0,
            target_group: 0,
            original_single_spell_card_target,
            moved_count: None,
            exiled_library_land: false,
            exiled_library_nonland: false,
            remembered_draws: None,
            remembered_damage_players: None,
            remembered_objects: None,
            discard_after: None,
            optional_wheel: None,
            color_sequence: None,
            discards: [0; 2],
            mana_payment: None,
            forage: None,
            repeated_life_choice: None,
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            discarded_card: None,
            milled_selection: None,
            library: None,
            selection: None,
            return_attachment: None,
            target_library: None,
            search: None,
            deferred_search_shuffle: false,
            search_result: None,
            exile_name_search: None,
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
                        .map(|f| {
                            if self.object_ref(f.object.card) == f.object {
                                self.state.objects[f.object.card as usize].controller
                            } else {
                                f.controller
                            }
                        })
                }),
                None,
            ],
            PlayerSet::EventPlayer => [
                Some(item.event_player.expect("event player must be captured")),
                None,
            ],
            PlayerSet::EventObjectController => [self.event_object_controller(item), None],
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
            PlayerSet::MostCreatures => {
                let mut counts = [0; 2];
                for (id, object) in self.state.objects.iter().enumerate() {
                    if object.zone == Zone::Battlefield && self.is_creature(id as CardId) {
                        counts[object.controller.index()] += 1;
                    }
                }
                let first = self.state.active;
                [first, first.other()].map(|player| {
                    (counts[player.index()] >= counts[player.other().index()]).then_some(player)
                })
            }
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

    /// Move chosen hands together, then draw in APNAP order.
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
                    ((object.zone == Zone::Hand
                        || (wheel.shuffle && object.zone == Zone::Graveyard))
                        && object.is_card()
                        && wheel.choices[object.owner.index()] == Some(true))
                    .then_some(id as CardId)
                })
                .collect();
            if wheel.shuffle {
                let events = self.move_objects_raw(&cards, Zone::Library);
                for player in [wheel.first, wheel.first.other()] {
                    if wheel.choices[player.index()] == Some(true) {
                        self.forget_library_top(player);
                        self.shuffle_library_cards(player);
                    }
                }
                self.finish_zone_events(&events, &[]);
            } else {
                self.discard_batch_with_after(&cards, None, Some(cause), None);
            }
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
            self.remember_opposing_face_down_creatures();
            self.queue_state_counter_triggers();
            if self.miracle_choice_pending() {
                return;
            }
            self.finish_power_difference_boost();
            self.finish_cascade();
            if self.advance_replacements(false) {
                return;
            }
            if self.finish_blight_copy() {
                // The creation continuation owns program resumption, including
                // when all entries and their replacements complete immediately.
                self.advance_replacements(true);
                return;
            }
            if self.advance_color_counter_sequence() {
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
            self.refresh_inherited_link_continuity();
            if self.advance_manifest_dread() {
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
            if self.advance_repeated_life_choice() {
                return;
            }
            if self.state.resolving.as_ref().unwrap().explore.is_some() {
                // A new placement batch must advance to an actual choice here.
                // A previous explore may have resumed this same program from
                // inside the action boundary's replacement settlement.
                if self.advance_explore() && self.state.replacements.is_none() {
                    return;
                }
                continue;
            }
            if self.advance_optional_wheel() {
                continue;
            }
            if self.resume_exile_continuation() {
                continue;
            }
            if self.advance_legendary_split_search() {
                return;
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
            if self.resume_exile_name_search() {
                return;
            }
            if self.state.resolving.as_ref().unwrap().connive.is_some() && self.advance_connive() {
                return;
            }
            // Each player chooses their discards privately in APNAP order.
            // Hand exhaustion completes that player's choices.
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
                    if self.finish_spell_with_resolution_exile(&item) {
                        // The selected resolution replacement already completed its exact move.
                    } else if item.cast.is_some_and(|c| c.face == CastFace::Adventure) {
                        self.finish_adventure(&item);
                    } else if item.cast.is_some_and(|c| c.face == CastFace::Omen) {
                        self.finish_omen(&item);
                    } else {
                        self.move_adventure_spell(&item, Zone::Graveyard);
                    }
                }
                self.prune_linked_exiles();
                if item.is_original_state_counter_trigger() {
                    self.queue_state_counter_triggers();
                }
                self.give_priority();
                return;
            };
            resolution.pc += 1;
            let mut item = resolution.item.clone();
            if !matches!(
                instruction,
                Instruction::Fight { .. }
                    | Instruction::IfTargetEnteredThisTurn { .. }
                    | Instruction::FightForExcess { .. }
                    | Instruction::GroupPowerDamage { .. }
                    | Instruction::GroupsPowerDamage { .. }
                    | Instruction::RevealTargetHandExileUntilSourceLeaves
                    | Instruction::MayAttachControlledEquipment { .. }
                    | Instruction::MayAttachAnyControlledEquipment { .. }
                    | Instruction::MayAttachControlledEquipmentUntilNextEndStep { .. }
                    | Instruction::MayDestroyAttachedEquipment { .. }
                    | Instruction::MayAttachSource { .. }
                    | Instruction::RequireTargetsBlocked { .. }
                    | Instruction::RestrictBlockerPower { .. }
                    | Instruction::AttachPriorSourceEquipment { .. }
                    | Instruction::AttachEquipmentByGroups { .. }
                    | Instruction::AttachEquipmentGroupsReflexive { .. }
                    | Instruction::ReturnAllTargetsToHand
                    | Instruction::DestroyAllTargets
                    | Instruction::ReturnGraveyardTargetsToHandOrBattlefield { .. }
                    | Instruction::ReturnGraveyardTargetsAnimated { .. }
                    | Instruction::ExileAllTargetsLinkedToSource
                    | Instruction::ExileAllTargets
                    // These aggregate operations consume every independently
                    // revalidated target role, regardless of SelectGroup.
                    | Instruction::MayIfTargetGroupsPresent { .. }
                    | Instruction::ExchangeControlTargets
                    | Instruction::ExchangeControlTargetGroup(_)
                    | Instruction::MakeTargetsTreasureWhileSourceRemains
                    | Instruction::BlinkFollowingPublicMove { all_groups: true }
                    | Instruction::DestroyTargetsThenSourceCountersIfOwnCreature { .. }
                    | Instruction::CopyTargetSpell { .. }
                    | Instruction::CopyTargetAbility { .. }
                    | Instruction::CopyTargetSpellOrAbility { .. }
                    | Instruction::CopyTargetAbilityX { .. }
                    | Instruction::DamageByGroup { .. }
                    | Instruction::GrantKeywordsToUnchosenTargets { .. }
                    | Instruction::LoyaltyDamage { .. }
                    | Instruction::PowerDamage { .. }
                    | Instruction::CreaturePowerDamageAny { .. }
                    | Instruction::CreaturePowerDamageBatch { .. }
                    | Instruction::SacrificeTarget { .. }
                    | Instruction::ScaledPowerDamage { .. }
                    | Instruction::CreateCopyTokenForPlayer { .. }
                    | Instruction::BecomeCopy(..)
                    | Instruction::ShuffleGraveyardTargetsIntoPlayerLibrary { .. }
                    | Instruction::ShuffleGraveyardTargetsIntoControllerLibrary { .. }
                    | Instruction::GainControl {
                        controller: PlayerSet::Targeted,
                        ..
                    }
                    | Instruction::GainControlRememberCount {
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
                    if self.start_discover(item.controller, u64::from(amount)) {
                        return;
                    }
                }
                Instruction::DiscoverScaled(amount) => {
                    let amount = u64::try_from(self.item_amount(amount, &item))
                        .capacity("discover value exhausted");
                    if self.start_discover(item.controller, amount) {
                        return;
                    }
                }
                Instruction::CounterTargetThenDiscover => {
                    if self.counter_target_then_discover(&item) {
                        return;
                    }
                }
                Instruction::ShuffleTargetThenDiscover => {
                    if self.shuffle_target_then_discover(&item) {
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
                Instruction::ExileTargetAndSearchItsController => {
                    self.exile_then_search_same_name(&item);
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
                Instruction::PartitionLibraryPiles(maximum)
                | Instruction::PartitionRevealedLibraryPiles(maximum) => {
                    self.start_library_piles(
                        item.controller,
                        maximum,
                        matches!(instruction, Instruction::PartitionRevealedLibraryPiles(_)),
                    );
                    return;
                }
                Instruction::KeepSharedCreatureTypes => {
                    if self.start_keep_shared_types(&item) {
                        return;
                    }
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
                Instruction::ReturnLinkedExileToHand => self.return_linked_exile(&item, Zone::Hand),
                Instruction::ReturnLinkedExileToBattlefield => {
                    self.return_linked_exile(&item, Zone::Battlefield);
                }
                Instruction::ExileTargetLibraryLinked(amount) => {
                    self.exile_target_library_linked(&item, amount)
                }
                Instruction::ExileTargetsLinkedToSource
                | Instruction::ExileAllTargetsLinkedToSource => {
                    self.exile_targets_linked(&item, false);
                }
                Instruction::ExileControllerLibraryTopLinked(amount)
                | Instruction::ExileControllerLibraryTopLinkedFaceDown(amount) => {
                    let face_down = matches!(
                        instruction,
                        Instruction::ExileControllerLibraryTopLinkedFaceDown(_)
                    );
                    self.exile_controller_library_top_linked(&item, amount, face_down);
                }
                Instruction::GrantLinkedExilePlay => self.grant_linked_exile_play(&item),
                Instruction::ExileGraveyardCaptureMutantCopy => {
                    self.exile_targets_linked(&item, true)
                }
                Instruction::ChooseLinkedCreatureReturn
                | Instruction::ChooseLinkedCreatureReturnWithCounters(_)
                | Instruction::ChooseLinkedCreatureAbilities
                | Instruction::CopyUntilDetachedFromLinkedExile
                | Instruction::CopyLinkedExiledAsMutant => {
                    use linked_exile_effects::LinkedExileMode;
                    let mode = match instruction {
                        Instruction::ChooseLinkedCreatureReturn => {
                            LinkedExileMode::ReturnCreature { plus_one: 0 }
                        }
                        Instruction::ChooseLinkedCreatureReturnWithCounters(plus_one) => {
                            LinkedExileMode::ReturnCreature { plus_one }
                        }
                        Instruction::ChooseLinkedCreatureAbilities => {
                            LinkedExileMode::BorrowCreatureAbilities
                        }
                        Instruction::CopyUntilDetachedFromLinkedExile => {
                            LinkedExileMode::CopyWhileAttached
                        }
                        _ => LinkedExileMode::CopyMutant,
                    };
                    if self.start_linked_exile_effect(&item, mode) {
                        return;
                    }
                }
                Instruction::ReturnOtherLinkedExileToHand => self.return_other_linked_exile(&item),
                Instruction::ExileOpponentUntilNonlandLinked => {
                    self.exile_opponent_until_nonland_linked(&item)
                }
                Instruction::CastOneLinkedDuringResolution
                | Instruction::CastOneOwnedLinkedPaid(_) => {
                    let owned_paid = matches!(instruction, Instruction::CastOneOwnedLinkedPaid(_));
                    if self.start_linked_exile_cast(&item, owned_paid) {
                        return;
                    }
                }
                Instruction::ReturnGraveyardNameTapped => self.return_graveyard_name_tapped(&item),
                Instruction::ReturnOneLinkedExileToHand => {
                    if !self.linked_exile_for_item(&item).is_empty() {
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
                    delayed.clear_initiating_event_references();
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
                Instruction::Meld(pair) => {
                    if self.begin_meld(&item, pair) {
                        return;
                    }
                }
                Instruction::ScheduleTurnTrigger {
                    event,
                    ability,
                    once,
                } => self.schedule_turn_callback(&item, event, ability, once, None),
                Instruction::ScheduleCastTriggerUntilEndOfNextTurn { ability } => {
                    self.schedule_turn_callback(
                        &item,
                        TurnCallbackEvent::SpellCast,
                        ability,
                        false,
                        Some(TurnCallbackExpiry::EndOfNextTurn),
                    );
                }
                Instruction::ScheduleNamedCombatDamageTrigger { ability } => {
                    if let Some(name) = resolution.chosen_card_name {
                        self.schedule_turn_callback(
                            &item,
                            TurnCallbackEvent::NamedCreatureCombatDamagePlayer(name),
                            ability,
                            false,
                            None,
                        );
                    }
                }
                Instruction::ScheduleTargetPlayerCombatDamageTrigger { ability } => {
                    if let Some(player) = self.program_players(PlayerSet::Targeted, &item).next() {
                        self.schedule_turn_callback(
                            &item,
                            TurnCallbackEvent::ControlledCreatureCombatDamagePlayer(player),
                            ability,
                            false,
                            None,
                        );
                    }
                }
                Instruction::AttackWithoutDefenderThisTurn => {
                    if let Some(source) = item.source {
                        self.allow_attack_without_defender(source.object);
                    }
                }
                Instruction::TargetsAttackWithoutDefenderThisTurn => {
                    for recipient in self.keyword_subjects(&item, Subject::Targets) {
                        self.allow_attack_without_defender(self.object_ref(recipient));
                    }
                }
                Instruction::CombatAndMainAfterCurrentMain(count) => {
                    self.add_combat_main_pairs(count)
                }
                Instruction::ScheduleAttackTriggerThisTurn { ability }
                | Instruction::DelayCreatureAttacksAlone { ability } => {
                    let mut delayed = item.clone();
                    delayed.id = self.next_stack_id();
                    delayed.kind = StackKind::TriggeredAbility { ability };
                    delayed.targets.clear();
                    delayed.clear_initiating_event_references();
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
                    delayed.target_type_slots = None;
                    self.state
                        .delayed_artifact_copies
                        .get_or_insert_default()
                        .push(delayed);
                }
                Instruction::GrantEmbalmTargets => self.grant_embalm_targets(&item),
                Instruction::CreateEmbalmedSourceCopy => self.create_embalmed_source_copy(&item),
                Instruction::CopySourceFromLinkedExile { base } => {
                    self.copy_source_from_linked_exile(&item, base)
                }
                Instruction::CopySourceFromGraveyardRetainingActivation {
                    blueprint_group,
                    keywords,
                } => self.copy_source_from_graveyard_retaining_activation(
                    &item,
                    blueprint_group,
                    keywords,
                ),
                Instruction::CopyEnteringPermanentThenAnimate(spec) => {
                    self.copy_entering_then_animate(&item, spec);
                }
                Instruction::CopyEnteringWithEquipDiscount(generic) => {
                    self.copy_entering_with_equip_discount(&item, generic);
                }
                Instruction::CreateCapturedArtifactCopy | Instruction::CreateCapturedCopy => {
                    if let Some(recipe) = item.captured_artifact_copy.as_deref() {
                        self.stage_copy_creation(item.controller, *recipe, 1);
                    }
                }
                Instruction::CopyNextSpell { ability }
                | Instruction::CopyNextInstantSorcery { ability }
                | Instruction::CopyNextExhaustAbility { ability }
                | Instruction::CopySpellsThisTurn { ability }
                | Instruction::DelayNextCreatureSpell { ability } => {
                    self.register_delayed_stack_trigger(&item, ability)
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
                Instruction::SuspendCapturedSpell { time } => {
                    self.suspend_captured_spell(&item, time)
                }
                Instruction::ResolveSuspendTick => self.resolve_suspend_tick(&item),
                Instruction::GrantCastRebound => self.grant_cast_rebound(&item),
                Instruction::GrantCastPlotOnResolution => self.grant_cast_plot_on_resolution(&item),
                Instruction::ExileTargetSpellPlotted => self.exile_target_spell_plotted(&item),
                Instruction::ResolveReboundCast => {
                    if self.resolve_rebound_cast(&item) {
                        return;
                    }
                }
                Instruction::ResolveSuspendCast => {
                    if self.resolve_suspend_cast(&item) {
                        return;
                    }
                }
                Instruction::CopyCapturedSpell { retarget, .. } => {
                    if self.copy_captured_spell_with_targets(&item, retarget) {
                        return;
                    }
                }
                Instruction::CopyCapturedSpellModified {
                    retarget,
                    modification,
                } => {
                    if self.copy_captured_spell_modified(&item, retarget, modification) {
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
                Instruction::MayCopyCastSpellWithKeywords(_) => {
                    if item
                        .copy_event
                        .as_ref()
                        .is_some_and(|event| event.amount > 0)
                    {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to: self.state.resolving.as_ref().unwrap().pc,
                        };
                        return;
                    }
                }
                Instruction::ChangeCastSpellTargets => {
                    if self.change_cast_spell_targets(&item) {
                        return;
                    }
                }
                Instruction::CopyRelatedTriggeredAbility => {
                    if self.copy_related_triggered_ability(&item) {
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
                Instruction::DamageTargetAndController {
                    damage,
                    controller_damage,
                } => {
                    let source = self.program_damage_source(&item);
                    let mut events: Vec<_> = item
                        .targets
                        .iter()
                        .filter(|&&bound| self.same_target_object(bound))
                        .map(|bound| DamageEvent {
                            preventable: true,
                            source,
                            target: bound.target,
                            amount: i128::from(damage),
                            combat: false,
                        })
                        .collect();
                    events.push(DamageEvent {
                        preventable: true,
                        source,
                        target: Target::Player(item.controller),
                        amount: i128::from(controller_damage),
                        combat: false,
                    });
                    self.apply_damage_batch(&events);
                }
                Instruction::DamageWithController {
                    damage,
                    controller_damage,
                } => self.damage_with_target_controller(
                    &item,
                    i128::from(damage),
                    i128::from(controller_damage),
                ),
                Instruction::DamageWithControllerScaled {
                    damage,
                    controller_damage,
                } => {
                    let amount = i128::try_from(self.item_amount(controller_damage, &item))
                        .capacity("damage amount exhausted");
                    self.damage_with_target_controller(&item, i128::from(damage), amount);
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
                Instruction::IfAnotherSpellCast { skip } => {
                    if self.item_amount(AmountValue::OtherSpellsCastThisTurn, &item) == 0 {
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
                Instruction::CounterOpponentsCounted => {
                    self.counter_opponents_counted(item.controller)
                }
                Instruction::CounterAndDrawForController { amount } => {
                    for bound in &item.targets {
                        if let Some((id, controller)) = self
                            .bound_stack_item(bound)
                            .map(|victim| (victim.id, victim.controller))
                        {
                            self.counter_stack_item(id, false);
                            self.draw_many(controller, u128::from(amount));
                        }
                    }
                }
                Instruction::DrawChosenCreatureType => {
                    let typ = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .chosen_creature_type
                        .expect("preceding creature-type choice");
                    let count = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.has_subtype(*id as CardId, typ.name())
                        })
                        .count();
                    self.draw_many(item.controller, count as u128);
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
                            delayed.clear_initiating_event_references();
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
                Instruction::SacrificeSourceAtEndCombat => {
                    if let Some(source) = item.source {
                        self.register_delayed_token_cleanup(
                            &item,
                            vec![source.object],
                            temporary_tokens::TokenCleanup::EndCombatSacrifice,
                        );
                    }
                }
                Instruction::ExileWarpedPermanent => self.exile_warped_permanent(&item),
                Instruction::ResolveSiegeDefeat => {
                    if self.resolve_siege_defeat(&item) {
                        return;
                    }
                }
                Instruction::Crew { .. } => self.resolve_crew(&item),
                Instruction::BecomeSaddled { .. } => self.resolve_saddle(&item),
                Instruction::MarkSaddled(subject) => self.mark_saddled_subject(&item, subject),
                Instruction::ChooseSaddleRiders(effect) => {
                    if self.start_saddle_rider_choice(&item, effect) {
                        return;
                    }
                }
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
                Instruction::PreventNextChosenSourceDamage { ability } => {
                    self.choose_source_prevention(ability);
                    return;
                }
                Instruction::CopyEntryBatchCreature => {
                    if !self.entry_batch_copy_options().is_empty() {
                        self.state.pending = Pending::EntryBatchCopy {
                            player: item.controller,
                        };
                        return;
                    }
                }
                Instruction::CreateTappedCopiesOfEntryBatch => {
                    let recipes = self
                        .entry_batch_copy_options()
                        .into_iter()
                        .map(|row| row.recipe)
                        .collect();
                    self.stage_copy_recipes(item.controller, recipes, 1, true);
                }
                Instruction::CopyControlledNonlandsFromTarget => {
                    self.copy_target_to_controlled_cohort(&item, true);
                }
                Instruction::CopyTargetToOtherControlledCreatures => {
                    self.copy_target_to_other_controlled_creatures(&item);
                }
                Instruction::ShuffleGraveyardTargetsIntoPlayerLibrary {
                    player_group,
                    cards_group,
                } => {
                    self.shuffle_graveyard_targets_into_player_library(
                        &item,
                        player_group,
                        cards_group,
                    );
                }
                Instruction::ShuffleGraveyardTargetsIntoControllerLibrary { cards_group } => {
                    self.shuffle_graveyard_targets_into_library(
                        &item,
                        item.controller,
                        cards_group,
                    );
                }
                Instruction::ShuffleGraveyardIntoLibraryCount { excluded_kinds } => {
                    self.shuffle_graveyard_into_library_count(item.controller, excluded_kinds);
                }
                Instruction::ShuffleYourLibrary => {
                    self.forget_library_top(item.controller);
                    self.shuffle_library_cards(item.controller);
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
                Instruction::DoubleTargetPower | Instruction::DoubleSourcePower => {
                    let subject = if instruction == Instruction::DoubleSourcePower {
                        Subject::Source
                    } else {
                        Subject::Targets
                    };
                    let affected: Vec<_> = self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .filter(|&id| self.is_creature(id))
                        .map(|id| (id, self.stats(id).power))
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
                Instruction::PutSourceOnLibraryBottom => self.put_source_on_library_bottom(&item),
                Instruction::TakeCreaturesUntilEndTurn => self.take_creatures(&item),
                Instruction::ChooseManaValueParityDestroyOthers
                | Instruction::ChooseManaValueParityDestroyCreatures => {
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
                Instruction::MayTapSourceReflexive { .. }
                | Instruction::MaySacrificeSourceReflexive { .. } => {
                    let skip_to = resolution.pc;
                    let can_pay =
                        if matches!(instruction, Instruction::MayTapSourceReflexive { .. }) {
                            self.exact_source_can_tap(&item)
                        } else {
                            self.exact_source_can_sacrifice(&item)
                        };
                    if can_pay {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                }
                Instruction::Reflexive { ability } => {
                    self.queue_reflexive_objects(&item, ability, item.reflexive_action_objects());
                }
                Instruction::ReflexiveForExcessDamage { ability } => {
                    if self.take_tracked_excess_damage() != 0 {
                        self.queue_reflexive(&item, ability, None);
                    }
                }
                Instruction::ReflexiveSacrificedPower { ability } => {
                    if item.sacrificed_power.is_some() {
                        self.queue_reflexive_objects(
                            &item,
                            ability,
                            item.reflexive_action_objects(),
                        );
                    }
                }
                Instruction::Modal(_)
                | Instruction::ModalOnce(_)
                | Instruction::ModalOncePerTurn(_)
                | Instruction::ModalUpToX { .. }
                | Instruction::ModalUpToStat { .. } => {
                    unreachable!("unselected modal ability executed")
                }
                Instruction::DestroyOthersIfSourcePowerTwenty => {
                    let power = self.item_amount(AmountValue::SourcePower, &item);
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
                Instruction::Explore(subject) => self.start_explore(&item, subject, 1),
                Instruction::ExploreRepeated { subject, amount } => {
                    let amount = self.item_amount(amount, &item);
                    self.start_explore(&item, subject, amount);
                }
                Instruction::ReturnToBattlefieldThenExplore(amount) => {
                    let amount = self.item_amount(amount, &item);
                    self.return_target_then_explore(&item, amount);
                }
                Instruction::ChooseRoomDoor {
                    selection,
                    operation,
                } => {
                    if self.start_room_choice(&item, selection, operation) {
                        return;
                    }
                }
                Instruction::IfRoomDoorAvailable {
                    selection,
                    operation,
                    skip,
                } => {
                    if !self.room_choice_available(&item, selection, operation) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::ManifestDread | Instruction::ManifestDreadRepeated(_) => {
                    let count = match instruction {
                        Instruction::ManifestDreadRepeated(amount) => {
                            self.item_amount(amount, &item)
                        }
                        _ => 1,
                    };
                    self.begin_manifest_dread(item.controller, count);
                }
                Instruction::ManifestDreadForTargetController => {
                    if let Some(player) = self
                        .program_players(PlayerSet::TargetController, &item)
                        .next()
                    {
                        self.begin_manifest_dread(player, 1);
                    }
                }
                Instruction::ManifestDreadForLinkedExileOwners => {
                    self.manifest_dread_for_linked_exile_owners(&item);
                }
                Instruction::CounterAndManifestForController => {
                    if let Some((id, player)) = item.targets.iter().find_map(|bound| {
                        self.bound_stack_item(bound)
                            .map(|spell| (spell.id, spell.controller))
                    }) {
                        self.counter_stack_item(id, false);
                        self.begin_manifest_dread(player, 1);
                    }
                }
                Instruction::AttachSourceToLastManifested => {
                    if let Some(target) = resolution
                        .manifest_sequence
                        .as_ref()
                        .and_then(|sequence| sequence.results.last().copied())
                    {
                        self.attach_equipment(
                            item.source.expect("equipment trigger has a source").object,
                            target,
                        );
                    }
                }
                Instruction::PutCountersOnManifested { kind, amount } => {
                    let amount = u64::try_from(self.item_amount(amount, &item))
                        .capacity("manifest counter amount exhausted");
                    let changes = self
                        .state
                        .resolving
                        .as_ref()
                        .and_then(|resolution| resolution.manifest_sequence.as_ref())
                        .into_iter()
                        .flat_map(|sequence| sequence.results.iter().copied())
                        .filter(|&object| {
                            self.on_battlefield(object.card)
                                && self.object_ref(object.card) == object
                        })
                        .map(|object| (object, Counters::of(kind, amount)))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::ManifestDreadWhenTargetDies => self.register_manifest_death(&item),
                Instruction::CloakShuffledRememberedExile => {
                    self.cloak_shuffled_remembered_exile(item.controller);
                }
                Instruction::CloakLibraryTop(players) => {
                    if let Some(owner) = self.program_players(players, &item).next() {
                        self.cloak_library_top(item.controller, owner);
                    }
                }
                Instruction::CloakFromHand => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::CloakFromHand,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::AttachSourceToEntryResult => {
                    if let (Some(source), Some(recipient)) =
                        (item.source, self.captured_entry_result())
                    {
                        self.attach_equipment(source.object, recipient);
                    }
                }
                Instruction::RevealFaceDownTargetAndMayTurnUp => {
                    let skip_to = resolution.pc + 1;
                    let can_turn =
                        self.first_bound_face_down_permanent(&item)
                            .is_some_and(|object| {
                                self.reveal_face_down_permanent(object)
                                    .is_some_and(Card::is_creature)
                                    && !manifest::face_up_prohibited(self, object)
                            });
                    if can_turn {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::TurnTargetFaceUp => {
                    if let Some(object) = self.first_bound_face_down_permanent(&item) {
                        self.turn_manifest_face_up(object, item.controller);
                    }
                }
                Instruction::ReturnDepartedPermanentFaceDown => {
                    self.return_departed_permanent_face_down(&item);
                }
                Instruction::MayTurnControlledCreatureFaceUp => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::TurnControlledCreatureFaceUp,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::TurnEntryResultFaceUp => {
                    if let Some(object) = self.captured_entry_result() {
                        self.turn_manifest_face_up(object, item.controller);
                    }
                }
                Instruction::TurnSourceFaceUpOrExile => {
                    let exiled = item.source.and_then(|source| {
                        if !self.on_battlefield(source.id)
                            || self.object_ref(source.id) != source.object
                            || self.turn_manifest_face_up(source.object, item.controller)
                        {
                            return None;
                        }
                        let event = self.move_object(source.id, Zone::Exile);
                        (event.moved && event.destination == Zone::Exile && event.was_card)
                            .then_some(event.after)
                    });
                    self.state.resolving.as_mut().unwrap().milled = exiled.into_iter().collect();
                }
                Instruction::LookAtOpposingFaceDownCreaturesUntilEndOfTurn => {
                    self.state
                        .manifest
                        .get_or_insert_with(Default::default)
                        .temporary_opposing_look[item.controller.index()] = true;
                }
                Instruction::PutCountersOnEventPermanent { kind, amount } => {
                    if let Some(object) = item.event_object.as_ref().map(|event| event.object)
                        && self.on_battlefield(object.card)
                        && self.object_ref(object.card) == object
                    {
                        self.effect_counters(
                            vec![(object, Counters::of(kind, amount))],
                            replacements::ResumeMode::Program,
                        );
                    }
                }
                Instruction::SkipIfFaceDownEntryOrTurnUp(skip) => {
                    if self.state.players[item.controller.index()]
                        .turn_facts
                        .face_down_entry_or_turn_up
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::DestroyAndCreateForController { token, amount } => {
                    let targets = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|b| self.same_target_object(*b))
                        .collect::<Vec<_>>();
                    self.destroy_and_create(targets, token, amount);
                }
                Instruction::DestroyTargetsRememberCreatureCards => {
                    self.destroy_targets_remember_creature_cards(&item);
                }
                Instruction::ReturnRememberedCardsForOwnEndStep => {
                    self.return_remembered_cards_for_own_end_step(&item);
                }
                Instruction::CollectEvidence { threshold, skip } => {
                    if self.start_resolution_evidence(threshold.into(), skip, false) {
                        return;
                    }
                }
                Instruction::CollectEvidenceX => {
                    self.state.pending = Pending::ResolutionX {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::ExileDepartedSourceCollectEvidenceReturnTapped(threshold) => {
                    if self.start_source_evidence_return(threshold.into()) {
                        return;
                    }
                }
                Instruction::CastLinkedEvidenceInstantOrSorcery { activation } => {
                    if self.offer_linked_evidence_cast(&item, activation) {
                        return;
                    }
                }
                Instruction::EvidenceWard(threshold) => {
                    if self.start_resolution_evidence(threshold.into(), 0, true) {
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
                Instruction::IfCastWithControlledSubtype { skip } => {
                    if !item
                        .cast
                        .is_some_and(|cast| cast.controlled_subtype_at_cast)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
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
                Instruction::IfSneakPaid { skip } => {
                    if !item
                        .cast
                        .is_some_and(|cast| cast.method == CastMethod::Sneak)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfKicked { skip } => {
                    if !item.cast.is_some_and(|c| c.kicker == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfWaterbendPaid { skip } => {
                    if !item.cast.is_some_and(|cast| cast.waterbend == Some(true)) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::MayPayWaterbend {
                    amount,
                    unpaid_skip,
                } => {
                    self.start_optional_resolution_payment(
                        item.controller,
                        waterbend::generic_cost(amount),
                        unpaid_skip,
                    );
                    self.enable_resolution_waterbend(amount);
                    return;
                }
                Instruction::CounterRelatedUnlessWaterbend(amount) => {
                    if self.start_resolution_payment(
                        &item,
                        waterbend::generic_cost(amount),
                        CounterSubject::Related,
                        false,
                        0,
                    ) {
                        self.enable_resolution_waterbend(amount);
                        return;
                    }
                }
                Instruction::ExileTargetGraveyardForWaterbend => {
                    let owner = self.program_players(PlayerSet::Targeted, &item).next();
                    if let Some(owner) = owner
                        && self.start_effect_choice(
                            &item,
                            EffectChoiceKind::ExileNoncreatureNonlandFromGraveyard { owner },
                            PlayerSet::Controller,
                            evidence::cost(0).filter,
                            1,
                            Some(0),
                        )
                    {
                        return;
                    }
                }

                Instruction::RememberTargetTapped => {
                    let tapped = item.targets.iter().any(|bound| {
                        self.same_target_object(*bound)
                            && matches!(bound.target, Target::Card(id)
                                if self.on_battlefield(id) && self.state.objects[id as usize].tapped)
                    });
                    self.state.resolving.as_mut().unwrap().moved_count = Some(u64::from(tapped));
                }
                Instruction::IfTargetEnteredThisTurn { group, skip } => {
                    if !item.targets.iter().any(|bound| {
                        bound.group == group
                            && matches!(bound.target, Target::Card(id) if self.on_battlefield(id))
                            && self.same_target_object(*bound)
                            && bound
                                .object
                                .is_some_and(|object| self.entered_this_turn(object))
                    }) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfTargetManaValueAtMost { maximum, skip } => {
                    let eligible = item.targets.iter().any(|&bound| {
                        self.same_target_object(bound)
                            && matches!(bound.target, Target::Card(id) if self.object_mana_value(id) <= maximum)
                    });
                    if !eligible {
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
                Instruction::IfSubjectSuspected { subject, skip } => {
                    if !self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .any(|id| self.is_suspected(id))
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfSubjectHasCounter {
                    subject,
                    kind,
                    skip,
                } => {
                    if !self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .any(|id| self.counter_count(id, kind) != 0)
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfSubjectSubtypeAny {
                    subject,
                    subtypes,
                    skip,
                } => {
                    if !subtypes
                        .iter()
                        .any(|name| self.subject_has_subtype(&item, subject, name))
                    {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfSubjectSubtype {
                    subject,
                    subtype,
                    skip,
                } => {
                    if !self.subject_has_subtype(&item, subject, subtype) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::If { guard, skip } => {
                    let source = item.source.expect("instruction requires an object source");
                    let holds = match guard {
                        StepGuard::Expression {
                            value: StatValue::SourceAttacking,
                            minimum,
                            maximum,
                        } if source.facts.battlefield
                            && (!self.on_battlefield(source.id)
                                || self.object_ref(source.id) != source.object) =>
                        {
                            // A departed source keeps the combat status captured
                            // at its last battlefield incarnation.
                            let attacking = i128::from(source.facts.attacking);
                            minimum.is_none_or(|min| attacking >= min)
                                && maximum.is_none_or(|max| attacking <= max)
                        }
                        _ => self.step_guard(
                            guard,
                            item.controller,
                            source.object,
                            item.source_tapped,
                        ),
                    };
                    if !holds {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::Skip(skip) => self.state.resolving.as_mut().unwrap().pc += skip,
                Instruction::AddRetainedManaScaled {
                    color,
                    amount,
                    retention,
                } => {
                    let mut mana = [0; 6];
                    mana[usize::from(color)] = u32::try_from(self.item_amount(amount, &item))
                        .capacity("mana representation exhausted");
                    self.add_quality_mana(
                        item.controller,
                        mana,
                        Some(mana_lots::Quality {
                            retention,
                            ..mana_lots::Quality::ORDINARY
                        }),
                    );
                }
                Instruction::AddCombatRed(amount) => {
                    self.add_combat_red(item.controller, u128::from(amount));
                    self.record_bend(item.controller, bending::BendKind::Fire);
                }
                Instruction::AddCombatRedScaled(value) => {
                    let amount = self.item_amount(value, &item);
                    self.add_combat_red(item.controller, amount);
                    self.record_bend(item.controller, bending::BendKind::Fire);
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
                Instruction::ShuffleHandAndGraveyardThenDraw(amount) => {
                    let mut choices = [Some(false); 2];
                    choices[item.controller.index()] = Some(true);
                    self.state.resolving.as_mut().unwrap().optional_wheel = Some(OptionalWheel {
                        first: item.controller,
                        choices,
                        amount,
                        next: 0,
                        running: true,
                        shuffle: true,
                    });
                }
                Instruction::BeginOptionalWheel(amount)
                | Instruction::BeginOptionalShuffleWheel(amount) => {
                    self.state.resolving.as_mut().unwrap().optional_wheel = Some(OptionalWheel {
                        first: self.state.active,
                        choices: [None; 2],
                        amount,
                        next: 0,
                        running: false,
                        shuffle: matches!(instruction, Instruction::BeginOptionalShuffleWheel(_)),
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
                Instruction::MayPayEnteringPower { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    let amount = self
                        .resolved_stat_value(&item, ResolvedStatValue::EnteringPower)
                        .unwrap_or(0)
                        .max(0);
                    if amount == 0 || self.state.players[item.controller.index()].life >= amount {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::PayEnteringPowerLifeAndCounters => {
                    let amount = self
                        .resolved_stat_value(&item, ResolvedStatValue::EnteringPower)
                        .unwrap_or(0)
                        .max(0);
                    self.lose_life(item.controller, amount);
                    if let Some(id) = self.entering_subject(&item) {
                        let mut counters = Counters::default();
                        counters.add(
                            CounterKind::PlusOne,
                            u64::try_from(amount).capacity("counter amount exhausted"),
                        );
                        self.effect_counters(
                            vec![(self.object_ref(id), counters)],
                            replacements::ResumeMode::Program,
                        );
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
                Instruction::MayPayEventPlayerLife { amount, skip } => {
                    if let Some(payer) = item.event_player
                        && self.start_optional_resolution_life(payer, i128::from(amount), skip)
                    {
                        return;
                    }
                    let resolution = self.state.resolving.as_mut().unwrap();
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    resolution.pc = skip_to;
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
                Instruction::ShuffleAttachedIntoOwnerLibrary => {
                    if let Some(id) = self.attached_permanent(&item) {
                        let owner = self.state.objects[id as usize].owner;
                        self.shuffle_cards_into_library(owner, &[id]);
                    }
                }
                Instruction::BlinkAttacking => self.blink_immediately_attacking(&item),
                Instruction::BlinkFollowingPublicMove { .. } => {
                    self.blink_following_public_move(&item)
                }
                Instruction::BlinkImmediately { counter, tapped } => {
                    self.blink_immediately(&item, counter, tapped)
                }
                Instruction::BlinkImmediatelyUnderController { tapped } => {
                    self.blink_immediately_under_controller(&item, tapped)
                }
                Instruction::BlinkCombatDamageDealer => {
                    if let Some(dealer) = self.live_combat_damage_dealer(&item) {
                        self.blink_battlefield_objects_immediately(&[dealer]);
                    }
                }
                Instruction::BlinkControlledCreaturesRepeated => {
                    let remaining = self
                        .state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .repeated_instruction_remaining;
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .repeated_instruction_remaining =
                        Some(remaining.unwrap_or_else(|| item.x.unwrap_or(0)));
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::BlinkControlledCreatures,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        0,
                        None,
                    ) {
                        return;
                    }
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .repeated_instruction_remaining = None;
                }
                Instruction::IfMayhemPaid { paid, skip } => {
                    if item.cast.is_some_and(|cast| cast.method.is_mayhem()) != paid {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
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
                Instruction::BlinkGroupTappedUntilEndStep => {
                    self.blink_group_until_end_step(&item, true, true, false, None)
                }
                Instruction::BlinkExiledTargetsUntilEndStep => {
                    self.blink_group_until_end_step(&item, true, false, true, None)
                }
                Instruction::ExileTargetAndCopyToSubtypeUntilEndStep { subtype } => {
                    self.exile_target_and_copy_to_subtype_until_end_step(&item, subtype)
                }
                Instruction::PreventPlayerDamageUntilNextTurn => {
                    self.prevent_player_damage_until_next_turn(&item)
                }
                Instruction::PreventControlledCreatureDamageThisTurn => self
                    .prevent_creature_damage_this_turn(
                        &item,
                        CreatureDamageShieldScope::ControlledCreatures,
                    ),
                Instruction::PreventCreatureDamageToControllerAndCreaturesThisTurn => self
                    .prevent_creature_damage_this_turn(
                        &item,
                        CreatureDamageShieldScope::PlayerAndCreaturesFromCreatures,
                    ),
                Instruction::DoubleDamageToDamagedPlayerUntilNextTurn => {
                    self.double_damage_to_damaged_player_until_next_turn(&item)
                }
                Instruction::IncreaseControlledNoncombatDamageThisTurn => {
                    self.increase_controlled_noncombat_damage_this_turn(&item)
                }
                Instruction::IncreaseOwnCreatureCountersThisTurn { kind, amount } => {
                    self.increase_own_creature_counters_this_turn(&item, kind, amount);
                }
                Instruction::BlinkUntilEndStep { under_owner } => {
                    self.blink_group_until_end_step(&item, under_owner, false, false, None)
                }
                Instruction::BlinkUntilEndStepWithCounter {
                    counter,
                    actual_exile_only,
                } => self.blink_group_until_end_step(
                    &item,
                    true,
                    false,
                    actual_exile_only,
                    Some((counter, 1)),
                ),
                Instruction::ReturnDeathCard(kind) => self.return_death_card(&item, kind),
                Instruction::ReturnDeathCardWithCounters {
                    under_controller,
                    tapped,
                    counter,
                    amount,
                } => self.return_death_card_with_counters(
                    &item,
                    under_controller,
                    tapped,
                    (counter, amount),
                ),
                Instruction::ReturnEventCardToHand => self.return_event_card_to_hand(&item),
                Instruction::ExileEventGraveyardCard
                | Instruction::ExileEventGraveyardCardLinked => {
                    let cards: Vec<_> = item
                        .event_object
                        .iter()
                        .flat_map(|event| event.graveyard_objects())
                        .filter_map(|object| {
                            let card = &self.state.objects[object.card as usize];
                            (card.zone == Zone::Graveyard
                                && card.generation == object.generation
                                && card.is_card())
                            .then_some(object.card)
                        })
                        .collect();
                    if !cards.is_empty() {
                        if matches!(instruction, Instruction::ExileEventGraveyardCardLinked) {
                            self.exile_objects_linked(&item, &cards);
                        } else {
                            let events = self.move_objects_raw(&cards, Zone::Exile);
                            self.finish_zone_events(&events, &[]);
                        }
                    }
                }
                Instruction::ExileEventPermanentPlayThisTurn => {
                    if let Some(id) = self.entering_subject(&item) {
                        let event = self.move_object(id, Zone::Exile);
                        if event.was_card && event.moved && event.destination == Zone::Exile {
                            for exiled in event.objects_after() {
                                self.grant_exile_play_permission(item.controller, exiled, false);
                            }
                        }
                    }
                }
                Instruction::PutDepartedSourceOnLibraryBottom => {
                    self.put_departed_source_on_library_bottom(&item);
                }
                Instruction::BottomDepartedSourceAndReturnLinkedExile => {
                    if self.put_departed_source_on_library_bottom(&item) {
                        self.return_linked_exile(&item, Zone::Hand);
                    }
                }
                Instruction::ReturnEventCardToBattlefieldTapped
                | Instruction::ResolveDelayedReturn => self.resolve_delayed_return(&item),
                Instruction::ReturnEventCardToBattlefieldTappedUnderControl => {
                    self.return_event_card_tapped_under_control(&item)
                }
                Instruction::ReturnDelayedBattlefieldObjectsToHand => {
                    self.return_delayed_battlefield_objects_to_hand(&item);
                }
                Instruction::RemoveDelayedObjectCounters { kind, amount } => {
                    self.remove_delayed_object_counters(&item, kind, u64::from(amount));
                }
                Instruction::CaptureDelayedReturnTarget => {
                    self.capture_delayed_return_target(&item)
                }
                Instruction::ScheduleStep {
                    step,
                    relation,
                    ability,
                }
                | Instruction::ScheduleStepThisTurn {
                    step,
                    relation,
                    ability,
                } => {
                    let expires_turn =
                        matches!(instruction, Instruction::ScheduleStepThisTurn { .. })
                            .then_some(self.state.turn);
                    self.schedule_delayed_step(&item, step, relation, ability, None, expires_turn);
                }
                Instruction::ScheduleStepWithExiledResult {
                    step,
                    relation,
                    ability,
                } => {
                    let objects = self.state.resolving.as_ref().unwrap().milled.clone();
                    self.schedule_delayed_step(&item, step, relation, ability, objects, None);
                }
                Instruction::PreventTargetActivationsWhileTapped => {
                    self.prevent_target_activations_until_untapped(&item);
                }
                Instruction::ReturnSourceFromFirstPublicZoneToHand => {
                    self.return_source_from_first_public_zone_to_hand(&item);
                }
                Instruction::ReturnSourceToHand(origin) => {
                    if let Some(source) = self.exact_source_in_zone(&item, origin) {
                        self.move_object(source, Zone::Hand);
                    }
                }
                Instruction::ReturnAttachmentGrantorToHand => {
                    if let Some(grant) = &item.attachment_trigger
                        && grant.granted
                        && self.on_battlefield(grant.grantor.card)
                        && self.object_ref(grant.grantor.card) == grant.grantor
                    {
                        self.move_object(grant.grantor.card, Zone::Hand);
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
                Instruction::PutHandSourceOntoBattlefield => {
                    if let Some(source) = item.source
                        && self.object_ref(source.id) == source.object
                        && self.state.objects[source.id as usize].zone == Zone::Hand
                        && self.state.objects[source.id as usize].owner == item.controller
                    {
                        let mut movement = self.entry_move(source.id, Zone::Battlefield, false);
                        movement.controller = item.controller;
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
                Instruction::ExchangeControlTargets => self.exchange_control_targets(&item, None),
                Instruction::ExchangeControlTargetGroup(group) => {
                    self.exchange_control_targets(&item, Some(group))
                }
                Instruction::TakeOpposingArtifactsUntilEndTurn => {
                    self.take_opposing_artifacts(&item)
                }
                Instruction::ReturnNewNonlandPermanents => {
                    let source = item.source.expect("return trigger has a source").object;
                    let ids: Vec<_> = self
                        .state
                        .entered_this_turn
                        .as_ref()
                        .into_iter()
                        .flat_map(|rows| rows.iter())
                        .map(|entry| entry.object)
                        .filter(|&r| {
                            r != source
                                && self.on_battlefield(r.card)
                                && self.object_ref(r.card) == r
                                && self.current_kinds(r.card) & 2 == 0
                        })
                        .map(|r| r.card)
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Hand);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::GainControlUntilEndTurn => self.gain_control_until_end_turn(&item),
                Instruction::GainControlConditionalVillainManaValue => {
                    self.gain_control_conditional_villain_mana_value(&item)
                }
                Instruction::GainControl {
                    subject,
                    controller,
                }
                | Instruction::GainControlRememberCount {
                    subject,
                    controller,
                } => {
                    let gained = self.gain_control(&item, subject, controller);
                    if matches!(instruction, Instruction::GainControlRememberCount { .. }) {
                        self.state.resolving.as_mut().unwrap().moved_count = Some(gained);
                    }
                }
                Instruction::IfSubjectControlledBy {
                    subject,
                    controller,
                    skip,
                } => {
                    let subjects = self.keyword_subjects(&item, subject);
                    let matches = self.program_players(controller, &item).any(|player| {
                        subjects.iter().any(|&id| {
                            self.on_battlefield(id)
                                && self.state.objects[id as usize].controller == player
                        })
                    });
                    if !matches {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::UnattachEventObject => self.unattach_event_object(&item),
                Instruction::ChooseGraveyardReturn {
                    maximum,
                    total_mana_value,
                } => {
                    if self.start_graveyard_return_selection(
                        &item,
                        u32::from(maximum),
                        Some(total_mana_value),
                    ) {
                        return;
                    }
                }
                Instruction::ChooseGraveyardReturnCount(count) => {
                    // The object pool cannot supply more candidates than this.
                    let maximum = u32::try_from(self.item_amount(count, &item)).unwrap_or(u32::MAX);
                    if self.start_graveyard_return_selection(&item, maximum, None) {
                        return;
                    }
                }
                Instruction::ChooseGraveyardReturnWithCounter(_)
                | Instruction::ChooseGraveyardReturnFromEither => {
                    if self.start_graveyard_return_selection(&item, 1, None) {
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
                Instruction::DestroyManaValueExactly { kinds, amount } => {
                    self.destroy_battlefield_mana_value(kinds, self.item_amount(amount, &item));
                }
                Instruction::DestroyCreaturesExceptChosenByPower(maximum) => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::KeepCreaturesByPowerForEachPlayer(maximum),
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::SacrificeCreaturesExceptEachPlayersChoice(_)
                | Instruction::ExileOpposingPermanentsChosenByEachPlayer(_) => {
                    let kind = match instruction {
                        Instruction::SacrificeCreaturesExceptEachPlayersChoice(maximum) => {
                            EffectChoiceKind::KeepCreaturesWithinTotalPower(maximum)
                        }
                        Instruction::ExileOpposingPermanentsChosenByEachPlayer(minimum) => {
                            EffectChoiceKind::ExileOpposingPermanentsByEachPlayer(minimum)
                        }
                        _ => unreachable!(),
                    };
                    if self.start_effect_choice(
                        &item,
                        kind,
                        PlayerSet::Each,
                        evidence::cost(0).filter,
                        0,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::DestroyCreaturesByToughness { maximum, remember } => {
                    let maximum =
                        i128::try_from(self.item_amount(maximum, &item)).unwrap_or(i128::MAX);
                    self.destroy_creatures_recording_graveyard(Some(maximum), remember, None);
                }
                Instruction::DestroyAllCreaturesRememberOwned => {
                    self.destroy_creatures_recording_graveyard(None, true, Some(item.controller));
                }
                Instruction::ReturnOwnedGraveyardNonlandPermanents => {
                    self.return_owned_graveyard_permanents(&item, 1 | 16 | 32 | 64 | 256, 2, false);
                }
                Instruction::ReturnOwnedGraveyardLandsTapped => {
                    self.return_owned_graveyard_permanents(&item, 2, 0, true);
                }
                Instruction::ReturnGraveyardArrivalBatch { tapped } => {
                    self.return_graveyard_arrival_batch(&item, tapped);
                }
                Instruction::ChooseOwnGraveyardLandReturn { tapped } => {
                    if self.start_graveyard_land_return(&item, tapped) {
                        return;
                    }
                }
                Instruction::SacrificeOtherCreaturesRememberGraveyard => {
                    self.sacrifice_other_creatures_remember_graveyard(&item);
                }
                Instruction::ReturnGraveyardCreaturesToOwnersExceptRemembered => {
                    self.return_graveyard_creatures_to_owners_except_remembered();
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
                } => self.return_battlefield_group(&item, filters, attacking, players, false),
                Instruction::ReturnBattlefieldCountOpposingControllers(filters) => {
                    self.return_battlefield_group(&item, filters, false, None, true);
                }
                Instruction::DamageBattlefield {
                    filters,
                    players,
                    amount,
                } => self.damage_battlefield_group(
                    &item,
                    filters,
                    players,
                    i128::from(amount),
                    None,
                    None,
                ),
                Instruction::DamageBattlefieldScaled { filters, amount } => {
                    let amount = i128::try_from(self.item_amount(amount, &item))
                        .capacity("damage amount exhausted");
                    self.damage_battlefield_group(&item, filters, None, amount, None, None);
                }
                Instruction::DamageBattlefieldAndTargets {
                    filters,
                    players,
                    amount,
                    target_amount,
                } => self.damage_battlefield_group(
                    &item,
                    filters,
                    players,
                    i128::from(amount),
                    None,
                    Some(target_amount),
                ),
                Instruction::DamageCreaturesExceptControlledTokens(amount) => {
                    self.damage_creatures_except_controlled_tokens(&item, amount);
                }
                Instruction::DamageCreaturesByOutlaw { outlaw, amount } => {
                    self.damage_creatures_by_outlaw(&item, outlaw, amount);
                }
                Instruction::DamageEachCreatureByDepartureManaValue => {
                    self.damage_each_creature_by_departure_mana_value(&item);
                }
                Instruction::DamageBattlefieldExceptSubtype {
                    filters,
                    excluded_subtype,
                    players,
                    amount,
                } => self.damage_battlefield_group(
                    &item,
                    filters,
                    players,
                    i128::from(amount),
                    Some(excluded_subtype),
                    None,
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
                Instruction::GrantDepartedAdventureUntilNextTurn => {
                    self.grant_departed_adventure_until_next_turn(&item)
                }
                Instruction::GrantCreatureGraveyardCast => {
                    self.grant_creature_graveyard_cast(item.controller)
                }
                Instruction::SolveSource => self.solve_case(
                    item.source
                        .expect("instruction requires an object source")
                        .object,
                    item.controller,
                ),
                Instruction::ReduceArtifactSpellCostsThisTurn(amount) => {
                    self.grant_artifact_spell_reduction(item.controller, amount)
                }
                Instruction::ReduceFaceDownSpellCostsThisTurn(amount) => {
                    self.grant_face_down_spell_reduction(item.controller, amount);
                }
                Instruction::ReduceColoredSpellCostsThisTurn { colors, amount } => {
                    let amount = self.item_amount(amount, &item);
                    self.grant_colored_spell_reduction(item.controller, colors, amount);
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
                Instruction::GrantExiledResultFlexiblePlay
                | Instruction::GrantExiledResultFlexibleCast => {
                    self.grant_exiled_result_flexible_play(
                        item.controller,
                        instruction == Instruction::GrantExiledResultFlexibleCast,
                    );
                }
                Instruction::GrantExiledResultCastUntilEndOfNextTurn => {
                    self.grant_remembered_exile_permission(
                        item.controller,
                        ExilePlayExpiry::EndOfNextTurn,
                        true,
                        false,
                    );
                }
                Instruction::GrantExiledResultFlexiblePlayUntilNextEndStep
                | Instruction::GrantExiledResultPlayUntilNextEndStep => {
                    self.grant_remembered_exile_permission(
                        item.controller,
                        ExilePlayExpiry::StartOfNextEndStep(item.controller),
                        false,
                        instruction == Instruction::GrantExiledResultFlexiblePlayUntilNextEndStep,
                    );
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
                Instruction::CastOpponentGraveyardFreeThenExile => {
                    if self.offer_opponent_graveyard_free_cast(item.controller) {
                        return;
                    }
                }
                Instruction::CastTargetedGraveyardFree => {
                    if self.offer_targeted_graveyard_cast(&item, None, true) {
                        return;
                    }
                }
                Instruction::CastTargetedGraveyardPaid {
                    spend_any_mana,
                    exile_on_graveyard,
                } => {
                    let paid = ResolutionPaidCast {
                        kinds: u8::MAX,
                        exile_instants_sorceries: false,
                        bottom_on_graveyard: false,
                        spend_any_mana,
                    };
                    if self.offer_targeted_graveyard_cast(&item, Some(paid), exile_on_graveyard) {
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
                Instruction::ExileTopMayCastPaid | Instruction::ExileTopMayCastPaidFrom(_) => {
                    // The result is set only when a child cast commits.
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    let library = match instruction {
                        Instruction::ExileTopMayCastPaidFrom(player) => {
                            self.program_players(player, &item).next()
                        }
                        _ => Some(item.controller),
                    };
                    if let Some(library) = library
                        && self.exile_top_offer_paid_cast(item.controller, library)
                    {
                        return;
                    }
                }
                Instruction::ExileTargetCopyCastPaid => {
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    if self.exile_targets_copy_offer_paid_cast(&item, Some(1)) {
                        return;
                    }
                }
                Instruction::ExileTargetCopyCastFree => {
                    if self.exile_target_copy_offer_free_cast(&item) {
                        return;
                    }
                }
                Instruction::ExileTargetsCopyCastPaid => {
                    if self.exile_targets_copy_offer_paid_cast(&item, None) {
                        return;
                    }
                }
                Instruction::IfOfferedCastDeclined { skip } => {
                    if self.state.resolving.as_ref().unwrap().moved_count == Some(0) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
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
                Instruction::ExileRandomGraveyardPlayThisTurn => {
                    self.exile_random_graveyard_play(item.controller)
                }
                Instruction::CopyRandomGraveyardPermanentUntilNonland => {
                    self.stage_random_graveyard_copy(item.controller);
                }
                Instruction::CounterRelatedUnlessPayLife(amount) => {
                    if self.start_resolution_life(&item, i128::from(amount)) {
                        return;
                    }
                }
                Instruction::CounterUnlessPayCosts {
                    subject,
                    costs,
                    exile,
                } => {
                    if self.start_resolution_costs(
                        &item,
                        item.controller,
                        Some(subject),
                        costs,
                        exile,
                        0,
                    ) {
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
                Instruction::DrawAndRepeatMillingIfSharedType => {
                    if self.milled_cards_share_type() {
                        let resolution = self.state.resolving.as_mut().unwrap();
                        let mill = resolution.pc.checked_sub(2).expect("preceding mill");
                        assert!(matches!(resolution.program[mill], Instruction::Mill(..)));
                        // Keep the next mill scheduled through any private draw
                        // or replacement choice. A successful test consumes at
                        // least two exact library cards into public zones.
                        resolution.pc = mill;
                        self.draw_many(item.controller, 1);
                    }
                }
                Instruction::MillThenReflexive { amount, ability } => {
                    let milled = self.mill_batch(
                        &[item.controller],
                        MillAmount::Fixed(amount),
                        item.controller,
                    );
                    if milled >= usize::from(amount) {
                        let item = self.state.resolving.as_ref().unwrap().item.clone();
                        self.queue_reflexive_objects(
                            &item,
                            ability,
                            item.reflexive_action_objects(),
                        );
                    }
                }
                Instruction::MayMill(_) | Instruction::MayMillAmount(_) => {
                    // CR701.17b forbids choosing to mill more than the library.
                    let amount = match instruction {
                        Instruction::MayMill(amount) => u128::from(amount),
                        Instruction::MayMillAmount(value) => self.item_amount(value, &item),
                        _ => unreachable!(),
                    };
                    let enough_cards = self.library_card_count(item.controller) as u128 >= amount;
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
                Instruction::SelectTargetPlayerMilled(spec) => {
                    if let Some(player) = self.program_players(PlayerSet::Targeted, &item).next()
                        && self.start_milled_selection_for(player, spec)
                    {
                        return;
                    }
                }
                Instruction::LookTopLandThenMayGraveyard => {
                    if self.start_top_land_selection(&item, library_selection::TopLandStage::Hand) {
                        return;
                    }
                }
                Instruction::LookTopMayPutLandOntoBattlefieldElseHand => {
                    if self.start_top_land_selection(
                        &item,
                        library_selection::TopLandStage::BattlefieldOrHand,
                    ) {
                        return;
                    }
                }
                Instruction::LookTopOneToHandThenLandsTapped(amount) => {
                    if self.start_top_hand_then_lands(&item, amount) {
                        return;
                    }
                }
                Instruction::RefillEmptyLibraryFromGraveyard(amount) => {
                    if self.refill_empty_library_from_graveyard(&item, amount) {
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
                Instruction::CastFilteredHandFree(_) => {
                    self.offer_hand_free_cast(item.controller, u128::MAX, None, false);
                    return;
                }
                Instruction::LookLibraryTopMayCastFreeOncePerTurn => {
                    if self.offer_library_top_free_once_per_turn(&item) {
                        return;
                    }
                }
                Instruction::CastHandFreeUpTo(maximum)
                | Instruction::CastHandPermanentFreeUpTo(maximum) => {
                    let permanent_spell_only =
                        matches!(instruction, Instruction::CastHandPermanentFreeUpTo(_));
                    if permanent_spell_only {
                        self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    }
                    self.offer_hand_free_cast(
                        item.controller,
                        self.item_amount(maximum, &item),
                        None,
                        permanent_spell_only,
                    );
                    return;
                }
                Instruction::CastHandFreeUpToOfType {
                    maximum,
                    permanent_type,
                } => {
                    self.offer_hand_free_cast(
                        item.controller,
                        self.item_amount(maximum, &item),
                        Some(permanent_type),
                        false,
                    );
                    return;
                }
                Instruction::BottomLookedLibraryRandom => self.finish_library_cast_look(),
                Instruction::ControlTargetPlayerNextTurn
                | Instruction::ControlTargetPlayerNextCombat => {
                    let combat_only = instruction == Instruction::ControlTargetPlayerNextCombat;
                    for player in self
                        .program_players(PlayerSet::Targeted, &item)
                        .collect::<Vec<_>>()
                    {
                        self.schedule_player_control(player, item.controller, combat_only);
                    }
                }
                Instruction::ExileAllCreaturesRemember => self.exile_all_creatures_remember(),
                Instruction::ExileAllCreaturesCount => self.exile_all_creatures_count(),
                Instruction::MayDeployFromHandOrGraveyard { filter, counter } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::DeployFromHandOrGraveyard { filter, counter },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::DeployHandPermanents => {
                    self.start_controller_hand_permanent_deployment(&item);
                    return;
                }
                Instruction::DeployControllerHandCreatures => {
                    self.start_controller_hand_creature_deployment(&item);
                    return;
                }
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
                Instruction::LookTargetHandMayDiscardThenDraw
                | Instruction::LookTargetHandMayDiscardNonland => {
                    let nonland_only =
                        matches!(instruction, Instruction::LookTargetHandMayDiscardNonland);
                    if self.start_private_hand_discard(&item, nonland_only) {
                        return;
                    }
                }
                Instruction::RevealTargetHandDiscardOrMoveExileToGraveyard => {
                    if self.start_reveal_discard_or_move_exile(&item) {
                        return;
                    }
                }
                Instruction::LookTargetHandMayExileUntilSourceLeaves => {
                    if self.start_private_hand_exile_until_source_leaves(&item) {
                        return;
                    }
                }
                Instruction::TargetPlayerExilesHandCard => {
                    if self.start_target_player_exiles_hand_card(&item) {
                        return;
                    }
                }
                Instruction::MayRevealHandSubtype { subtype, .. } => {
                    self.start_optional_hand_reveal(&item, subtype);
                    return;
                }
                Instruction::RevealTargetHandSubsetDiscard(amount) => {
                    if self.start_hand_subset_discard(&item, amount) {
                        return;
                    }
                }
                Instruction::ExileUntilNonlandCastOrHand { maximum_mana_value } => {
                    if self.start_exile_until_nonland_cast_or_hand(&item, |_| {
                        maximum_mana_value.checked_add(1)
                    }) {
                        return;
                    }
                }
                Instruction::ExileUntilNonlandCastBelowOrHand(value) => {
                    if self.start_exile_until_nonland_cast_or_hand(&item, |game| {
                        u64::try_from(game.item_amount(value, &item)).ok()
                    }) {
                        return;
                    }
                }
                Instruction::MayDiscardNonlandReflexive(ability)
                | Instruction::MayDiscardAnyReflexive(ability) => {
                    let any_number = matches!(instruction, Instruction::MayDiscardAnyReflexive(_));
                    self.start_optional_reflexive_discard(&item, ability, any_number);
                    return;
                }
                Instruction::RevealTargetHandMayExileLinked => {
                    if self.start_optional_revealed_hand_link(&item) {
                        return;
                    }
                }
                Instruction::CreateSizedTokensForLinkedExileOwners { token } => {
                    self.create_sized_tokens_for_linked_exile_owners(&item, token);
                }
                Instruction::ExileDamageEventLibraryCastFree => {
                    if self.exile_damage_event_library_cast_free(&item) {
                        return;
                    }
                }
                Instruction::ExileTopCastFree { amount, kinds } => {
                    let amount = self.item_amount(amount, &item);
                    if self.exile_top_offer_free_cast(
                        item.controller,
                        item.controller,
                        amount,
                        None,
                        kinds,
                    ) {
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
                Instruction::RevealTargetHandChoose { kinds, exile } => {
                    if self.start_revealed_hand_kind_choice(&item, kinds, exile) {
                        return;
                    }
                }
                Instruction::RevealTargetHandExileNonland
                | Instruction::RevealTargetHandExileNonlandCastIfGift
                | Instruction::RevealTargetHandExileNonlandReward { .. } => {
                    let kind = match instruction {
                        Instruction::RevealTargetHandExileNonlandReward { maximum, token } => {
                            hand_choices::HandChoiceKind::ExileRevealedNonlandReward {
                                maximum,
                                token,
                            }
                        }
                        Instruction::RevealTargetHandExileNonlandCastIfGift
                            if item.gift_context().is_some() =>
                        {
                            hand_choices::HandChoiceKind::ExileRevealedNonlandCast
                        }
                        _ => hand_choices::HandChoiceKind::ExileRevealedNonland,
                    };
                    if self.start_revealed_hand_exile_nonland(&item, kind) {
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
                Instruction::RecoverMilled(filter)
                | Instruction::RecoverMilledOrSkip { filter, .. } => {
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
                Instruction::TapAttached | Instruction::UntapAttached => {
                    if let Some(id) = self.attached_permanent(&item) {
                        if instruction == Instruction::UntapAttached {
                            self.untap_objects(&[id], untap_rules::UntapCause::Effect);
                        } else {
                            self.tap_objects_as(item.controller, &[id]);
                        }
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
                    if self.may_attach_controlled_equipment(
                        &item,
                        recipient,
                        return_attachment::ControlledEquipmentSelection::One(required_subtype),
                    ) {
                        return;
                    }
                }
                Instruction::MayAttachAnyControlledEquipment { recipient } => {
                    if self.may_attach_controlled_equipment(
                        &item,
                        recipient,
                        return_attachment::ControlledEquipmentSelection::Any,
                    ) {
                        return;
                    }
                }
                Instruction::MayAttachControlledEquipmentUntilNextEndStep {
                    recipient,
                    ability,
                } => {
                    if self.may_attach_controlled_equipment(
                        &item,
                        recipient,
                        return_attachment::ControlledEquipmentSelection::UntilNextEndStep(ability),
                    ) {
                        return;
                    }
                }
                Instruction::AttachPriorSourceEquipment { recipient } => {
                    if self.attach_prior_source_equipment(&item, recipient) {
                        return;
                    }
                }
                Instruction::MayAttachReflexiveEquipment { recipient_subtype } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::ReflexiveEquipmentRecipient {
                            subtype: recipient_subtype,
                        },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
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
                Instruction::MayDestroyAttachedEquipment { recipient } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::AttachedEquipmentToDestroy { recipient },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
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
                            for (index, ability) in source.trigger_templates() {
                                if ability.condition.event()
                                    == TriggerCondition::BecomesClassLevel(level)
                                {
                                    self.queue_trigger(&source, index);
                                }
                            }
                        }
                    }
                }
                Instruction::AttachSourceReflexive { ability } => {
                    if let Some(target) = item.targets.iter().find_map(|bound| bound.object) {
                        let source = item.source.expect("equipment ability has a source").object;
                        if self.equipment_can_attach(source, target)
                            && self.state.objects[source.card as usize].attached_to != Some(target)
                            && self.attach_equipment(source, target)
                        {
                            self.queue_reflexive(&item, ability, Some(target));
                        }
                    }
                }
                Instruction::AttachSourceToEntering => {
                    if let Some(target) = item.entering_object {
                        let source = item.source.expect("equipment ability has a source").object;
                        self.attach_equipment(source, target);
                    }
                }
                Instruction::ExileAttached => {
                    if let Some(target) = self.attached_permanent(&item) {
                        self.move_object(target, Zone::Exile);
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
                Instruction::MayAttachSource { recipient, skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if self.new_source_attachment_pair(&item, recipient).is_some() {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::AttachSourceToControlledCreature => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::EquipmentRecipient,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        None,
                    ) {
                        return;
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
                Instruction::DrawTargetControllerIfUndamaged(amount) => {
                    let player = self.state.resolving.as_ref().and_then(|r| {
                        r.target_card_facts
                            .as_ref()?
                            .iter()
                            .find(|f| f.group == r.target_group && !f.damaged)
                            .map(|f| f.controller)
                    });
                    if let Some(player) = player {
                        self.draw_many(player, amount.into());
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
                Instruction::DestroyAllOfChosenPermanentType(types) => {
                    assert!(!types.is_empty(), "empty resolution permanent type choice");
                    self.state.pending = Pending::ResolutionPermanentType {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::ChooseNumberDestroyPowerAtLeast { minimum, maximum } => {
                    assert!(minimum <= maximum, "empty resolution number range");
                    self.state.pending = Pending::PowerThreshold {
                        player: item.controller,
                        minimum,
                        maximum,
                    };
                    return;
                }
                Instruction::DestroyCreaturesWithPowerAtLeast(minimum) => {
                    self.destroy_creatures_by_power(minimum);
                }
                Instruction::ChooseColor => {
                    self.state.resolving.as_mut().unwrap().chosen_color = None;
                    self.state.pending = Pending::ResolutionColor {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::ChooseSourceColorAndHexproof => {
                    if item.source.is_some_and(|source| {
                        self.on_battlefield(source.id)
                            && self.object_ref(source.id) == source.object
                    }) {
                        self.state.pending = Pending::ResolutionColor {
                            player: item.controller,
                        };
                        return;
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
                Instruction::ChooseSourceCounterKind(choices) => {
                    if let Some(source) = item.source.filter(|source| {
                        self.on_battlefield(source.id)
                            && self.object_ref(source.id) == source.object
                    }) {
                        assert!(!choices.is_empty(), "counter kind choice must have options");
                        self.state.pending = Pending::SourceCounterKind {
                            player: item.controller,
                            source: source.object,
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
                Instruction::CreatureStatsUntilNextTurn {
                    filter,
                    power,
                    toughness,
                } => {
                    self.creature_stats_until_next_turn(&item, filter, power, toughness);
                }
                Instruction::CreateTokenIfControllerSacrificed(token) => {
                    if self.state.resolving.as_ref().unwrap().moved_count == Some(1) {
                        self.create_tokens(item.controller, token, 1, false);
                    }
                }
                Instruction::MayBlightCopyAttacking => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::BlightCost(1),
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .choice
                            .as_mut()
                            .unwrap()
                            .blight_copy_attacking = true;
                        return;
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
                Instruction::SacrificeScaled {
                    players,
                    filter,
                    amount,
                } => {
                    if self.start_scaled_sacrifice(&item, players, filter, amount) {
                        return;
                    }
                }
                Instruction::MaySacrificeForGrantedReflexive { filter, .. } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::Sacrifice,
                        PlayerSet::Controller,
                        filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::MaySacrificeForManaValue { filter, skip } => {
                    self.state.resolving.as_mut().unwrap().sacrificed_mana_value = None;
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::Sacrifice,
                        PlayerSet::Controller,
                        filter,
                        1,
                        Some(skip),
                    ) {
                        return;
                    }
                }
                Instruction::MaySacrificeAnyNumber(filter)
                | Instruction::MaySacrificeAnyNumberForPower(filter) => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.moved_count = Some(0);
                    let kind =
                        if matches!(instruction, Instruction::MaySacrificeAnyNumberForPower(_)) {
                            resolution.sacrificed_power = None;
                            EffectChoiceKind::SacrificeSelectionForPower
                        } else {
                            EffectChoiceKind::SacrificeSelection
                        };
                    if self.start_effect_choice(&item, kind, PlayerSet::Controller, filter, 0, None)
                    {
                        return;
                    }
                }
                Instruction::ChooseOpposingCreatureForDeparture {
                    other_power,
                    other_toughness,
                } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::LinkedCreature {
                            other_power,
                            other_toughness,
                        },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::CreateNonlegendaryCopyRecordAura(subject) => {
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        recipe.overrides = Some(CopyOverrides {
                            legendary: Some(false),
                            ..recipe.overrides.unwrap_or(CopyOverrides::NONE)
                        });
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe,
                            1,
                            replacements::TokenCompletion::RecordCreatedAura,
                        );
                    }
                }
                Instruction::CreateReflectionCopy(subject) => {
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        // Reflection is a creature subtype (also allowed on Kindred).
                        if recipe.kinds() & (1 | 128) != 0 {
                            recipe = recipe.with_exceptions(CopyExceptions {
                                reflection: true,
                                ..CopyExceptions::NONE
                            });
                        }
                        let original = recipe.overrides.unwrap_or(CopyOverrides::NONE);
                        recipe.overrides = Some(CopyOverrides {
                            legendary: Some(false),
                            keywords: original.keywords.union(Keywords::HASTE),
                            ..original
                        });
                        self.stage_copy_creation(item.controller, recipe, 1);
                    }
                }
                Instruction::SacrificeControlledSubtype(subtype) => {
                    let sacrifices: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(index, object)| {
                            let id = index as CardId;
                            (object.zone == Zone::Battlefield
                                && object.controller == item.controller
                                && self.has_subtype(id, subtype))
                            .then_some((item.controller, id))
                        })
                        .collect();
                    self.sacrifice_batch(&sacrifices);
                }
                Instruction::CreateNonlegendaryCopyTokens { subject, amount }
                | Instruction::CreateLegendaryCopyTokens { subject, amount } => {
                    if let Some(mut recipe) = self.recipe_for_subject(&item, subject) {
                        let legendary =
                            matches!(instruction, Instruction::CreateLegendaryCopyTokens { .. });
                        recipe.overrides = Some(CopyOverrides {
                            legendary: Some(legendary),
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
                Instruction::CreateTokensAttacking { token, .. }
                | Instruction::CreateTokensAttackingScaled { token, .. } => {
                    let amount = match instruction {
                        Instruction::CreateTokensAttacking { amount, .. } => u128::from(amount),
                        Instruction::CreateTokensAttackingScaled { amount, .. } => {
                            self.item_amount(amount, &item)
                        }
                        _ => unreachable!(),
                    };
                    let attacking = self.attacking_return_combat(item.controller);
                    self.stage_token_creation(
                        item.controller,
                        token,
                        amount,
                        true,
                        if attacking {
                            replacements::TokenCompletion::Attacking
                        } else {
                            replacements::TokenCompletion::None
                        },
                    );
                }
                Instruction::CreateLinkedExileAttackingCopies => {
                    self.create_linked_exile_attacking_copies(&item)
                }
                Instruction::CreateHastyTokensDestroyedAtEnd { token, amount } => {
                    let count = self.item_amount(amount, &item);
                    self.stage_token_creation(
                        item.controller,
                        token,
                        count,
                        false,
                        replacements::TokenCompletion::TemporaryTokens(Box::new(
                            temporary_tokens::TokenInstructions {
                                item: item.clone(),
                                keywords: Keywords::HASTE,
                                attack: temporary_tokens::TokenAttack::None,
                                cleanup: temporary_tokens::TokenCleanup::NextEndStepDestroy,
                            },
                        )),
                    );
                }
                Instruction::CreateTokensThenCleanup {
                    token,
                    amount,
                    attacking,
                    cleanup,
                } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        attacking,
                        replacements::TokenCompletion::TemporaryTokens(Box::new(
                            temporary_tokens::TokenInstructions {
                                item: item.clone(),
                                keywords: Keywords::NONE,
                                attack: if attacking
                                    && self.attacking_return_combat(item.controller)
                                {
                                    temporary_tokens::TokenAttack::DefendingPlayer
                                } else {
                                    temporary_tokens::TokenAttack::None
                                },
                                cleanup,
                            },
                        )),
                    );
                }
                Instruction::CreateTemporaryTokens {
                    token,
                    amount,
                    keywords,
                }
                | Instruction::CreateTemporaryTokensThenCleanup {
                    token,
                    amount,
                    keywords,
                    ..
                } => {
                    let cleanup = match instruction {
                        Instruction::CreateTemporaryTokensThenCleanup { cleanup, .. } => cleanup,
                        _ => temporary_tokens::TokenCleanup::NextEndStepSacrifice,
                    };
                    self.stage_token_creation(
                        item.controller,
                        token,
                        self.item_amount(amount, &item),
                        false,
                        replacements::TokenCompletion::TemporaryTokens(Box::new(
                            temporary_tokens::TokenInstructions {
                                item: item.clone(),
                                keywords,
                                attack: temporary_tokens::TokenAttack::None,
                                cleanup,
                            },
                        )),
                    );
                }
                Instruction::CreateCopyWithDelayedSacrifice => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_temporary_copy_creation(&item, recipe);
                    }
                }
                Instruction::CreateCopyWithExceptionsAndDelayedSacrifice(exceptions) => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe.with_exceptions(exceptions),
                            1,
                            replacements::TokenCompletion::TemporaryTokens(Box::new(
                                temporary_tokens::TokenInstructions {
                                    item: item.clone(),
                                    keywords: Keywords::NONE,
                                    attack: temporary_tokens::TokenAttack::None,
                                    cleanup: temporary_tokens::TokenCleanup::NextEndStepSacrifice,
                                },
                            )),
                        );
                    }
                }
                Instruction::CreateHastyCopyWithDelayedSacrifice => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_temporary_copy_creation(&item, recipe);
                    }
                }
                Instruction::CreateHastyCopiesWithDelayedSacrifice(amount) => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        let count = self.item_amount(amount, &item);
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe,
                            count,
                            replacements::TokenCompletion::TemporaryCopy(Box::new(item.clone())),
                        );
                    }
                }
                Instruction::CreateHastyCopyWithSagaLore { maximum } => {
                    if let Some(recipe) = self.recipe_for_subject(&item, Subject::Targets) {
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe,
                            1,
                            replacements::TokenCompletion::CopiedSagaLore { maximum },
                        );
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
                Instruction::CopyTargetForSourceIncarnation => {
                    if let (Some(source), Some(recipe)) = (
                        item.source,
                        self.recipe_for_subject(&item, Subject::Targets),
                    ) {
                        self.apply_temporary_copy(
                            source.object,
                            recipe,
                            CopyDeadline::WhileSource {
                                source: source.object,
                            },
                        );
                    }
                }
                Instruction::CopySourceFromRememberedPermanent => {
                    let blueprint = self.live_remembered_permanents().next();
                    if let (Some(source), Some(blueprint)) = (item.source, blueprint) {
                        let recipe = self.copiable_recipe(blueprint);
                        self.apply_temporary_copy(source.object, recipe, CopyDeadline::EndOfTurn);
                    }
                }
                Instruction::CreateCopyOfRememberedPermanent => {
                    let recipe = self
                        .live_remembered_permanents()
                        .next()
                        .map(|object| self.copiable_recipe(object));
                    if let Some(recipe) = recipe {
                        self.stage_copy_creation(item.controller, recipe, 1);
                    }
                }
                Instruction::MayCopyLinkedExiledCreatureUntilEndOfTurn => {
                    if self.start_linked_exile_effect(
                        &item,
                        linked_exile_effects::LinkedExileMode::CopyCreatureUntilEndOfTurn,
                    ) {
                        return;
                    }
                }
                Instruction::CopySourceFromRememberedExiledCreature => {
                    if let Some((source, recipe)) = self.remembered_exiled_creature_copy(&item) {
                        self.apply_temporary_copy(source, recipe, CopyDeadline::EndOfTurn);
                    }
                }
                Instruction::PutRememberedPermanentCounters { kind, amount } => {
                    let changes = self
                        .live_remembered_permanents()
                        .map(|object| (object, Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::AnimateRememberedPermanents(animation) => {
                    let objects: Vec<_> = self.live_remembered_permanents().collect();
                    for object in objects {
                        self.change_object_form(
                            object,
                            animation,
                            characteristic_layers::AnimationTypeMode::AddCreature,
                        );
                    }
                }
                Instruction::CopyTargetsFromCastSpell => self.copy_targets_from_cast_spell(&item),
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
                }
                | Instruction::PutPermanentCountersWithColors {
                    filters,
                    kind,
                    amount,
                    ..
                }
                | Instruction::PutLegendaryPermanentCountersScaled {
                    filters,
                    kind,
                    amount,
                }
                | Instruction::PutNamedPermanentCountersScaled {
                    filters,
                    kind,
                    amount,
                    ..
                } => {
                    self.put_permanent_counters_scaled(&item, filters, kind, amount, |game, id| {
                        match instruction {
                            Instruction::PutPermanentCountersWithColors { colors, .. } => {
                                ability_world::CharacteristicWorld::colors(game, id) == colors
                            }
                            Instruction::PutLegendaryPermanentCountersScaled { .. } => {
                                ability_world::CharacteristicWorld::legendary(game, id)
                            }
                            Instruction::PutNamedPermanentCountersScaled { name, .. } => {
                                crate::card_names::same_name(game.object_name(id), name)
                            }
                            _ => true,
                        }
                    });
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
                Instruction::CreateCopyTokenUntilNextEndStep {
                    subject,
                    amount,
                    ability,
                } => {
                    if let Some(recipe) = self.recipe_for_subject(&item, subject) {
                        self.stage_copy_creation_inner(
                            item.controller,
                            recipe,
                            u128::from(amount),
                            replacements::TokenCompletion::TemporaryTokens(Box::new(
                                temporary_tokens::TokenInstructions {
                                    item: item.clone(),
                                    keywords: Keywords::NONE,
                                    attack: temporary_tokens::TokenAttack::None,
                                    cleanup: temporary_tokens::TokenCleanup::NextEndStepExile {
                                        ability,
                                    },
                                },
                            )),
                        );
                    }
                }
                Instruction::CreateCopyTokensScaled { subject, amount } => {
                    if let Some(recipe) = self.recipe_for_subject(&item, subject) {
                        let amount = self.item_amount(amount, &item);
                        self.stage_copy_creation(item.controller, recipe, amount);
                    }
                }
                Instruction::CreateCopiesOfEachTarget(value) => {
                    let amount = self.item_amount(value, &item);
                    self.copy_each_target(&item, amount);
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
                Instruction::SacrificeSourceByController => {
                    if let Some(source) = item.source
                        && self.on_battlefield(source.id)
                        && self.object_ref(source.id) == source.object
                    {
                        let controller = self.state.objects[source.id as usize].controller;
                        self.sacrifice_batch(&[(controller, source.id)]);
                    }
                }
                Instruction::SacrificeSourceOrSkip { skip } => {
                    if !self.try_sacrifice_exact_source(&item) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::SacrificeSourceReflexive { ability } => {
                    self.sacrifice_source_reflexive(&item, ability);
                }
                Instruction::ExileAndCopyNightmare => self.exile_and_copy_nightmare(&item),
                Instruction::ExileGraveyardTargetsCopyUntilOwnEndStep => {
                    self.exile_graveyard_targets_copy_until_own_end_step(&item);
                }
                Instruction::ExileAndCopyCreatureAs {
                    base,
                    color,
                    subtype,
                } => self.exile_and_copy_creature_as(&item, base, color, subtype),
                Instruction::ExileOtherNightmares => self.exile_other_nightmares(item.controller),
                Instruction::DamageDivided { .. } | Instruction::DamageDividedScaled(_) => {
                    self.damage_divided(&item)
                }
                Instruction::DistributeCounters { kind, .. }
                | Instruction::DistributeCountersX { kind } => {
                    self.distribute_counters(&item, kind)
                }
                Instruction::DistributeCountersAmongPermanents {
                    total,
                    kind,
                    filter,
                } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::CounterDistribution { kind, filter },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        total,
                        None,
                    ) {
                        return;
                    }
                }
                Instruction::DamageByGroup { first, other } => {
                    self.damage_by_group(&item, first, other)
                }
                Instruction::DamageForExcess(amount) => self.damage_for_excess(&item, amount),
                Instruction::GainExcessDamageLife => {
                    let amount = self.take_tracked_excess_damage();
                    self.gain_life(item.controller, i128::from(amount));
                }
                Instruction::AddExcessDamageMana(color) => {
                    let amount = self.take_tracked_excess_damage();
                    let mut mana = [0; 6];
                    mana[color.index()] =
                        u32::try_from(amount).capacity("mana representation exhausted");
                    self.add_mana(item.controller, mana);
                }
                Instruction::RetainManaColorThisTurn(color) => {
                    self.retain_mana_color_until_cleanup(item.controller, color);
                }
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
                Instruction::CreateExcessTokens(token)
                | Instruction::CreateTappedExcessTokens(token) => {
                    self.create_excess_tokens(
                        &item,
                        token,
                        matches!(instruction, Instruction::CreateTappedExcessTokens(_)),
                    );
                }
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
                    if self.copy_target_ability(&item, group, 1) {
                        return;
                    }
                }
                Instruction::CopyTargetSpellOrAbility { group } => {
                    if self.copy_target_spell_or_ability(&item, group) {
                        return;
                    }
                }
                Instruction::CopyTargetAbilityX { group } => {
                    if self.copy_target_ability(&item, group, u64::from(item.x.unwrap_or(0))) {
                        return;
                    }
                }
                Instruction::CopyCapturedActivatedAbility => {
                    if self.copy_captured_activated_ability(&item) {
                        return;
                    }
                }
                Instruction::IfCoinFlipWon { skip } => {
                    if self.flip_coins(item.controller, 1) == 0 {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::FlipCoinForCapturedSpell { win, loss } => {
                    self.flip_coin_for_captured_spell(&item, win, loss);
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
                Instruction::LoseHalfLifeRoundedUp(players) => {
                    for player in self.program_players(players, &item) {
                        // CR107.1b: negative calculated loss is zero. Avoid
                        // overflowing at i128::MAX when rounding an odd half up.
                        let life = self.state.players[player.index()].life.max(0);
                        self.lose_life(player, life / 2 + life % 2);
                    }
                }
                Instruction::ChooseHalf { .. } | Instruction::ChooseHalfRoundedUp { .. } => {
                    let (players, kind, filter, round_up) = match instruction {
                        Instruction::ChooseHalf { kind } => {
                            (PlayerSet::Each, kind, evidence::cost(0).filter, false)
                        }
                        Instruction::ChooseHalfRoundedUp {
                            players,
                            kind,
                            filter,
                        } => (players, kind, filter, true),
                        _ => unreachable!(),
                    };
                    if self.start_half_choice(&item, players, kind, filter, round_up) {
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
                Instruction::AdditionalCombat | Instruction::AdditionalLandCreatureCombat => {
                    self.add_combat(instruction == Instruction::AdditionalLandCreatureCombat);
                }
                Instruction::AdditionalUpkeepsAfterCombat(amount) => {
                    self.add_upkeeps_after_combat(item.controller, self.item_amount(amount, &item));
                }
                Instruction::AdditionalEndStepIfFirst => self.add_end_step_if_first(),
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
                Instruction::RequireSubjectCombat {
                    subject,
                    requirement,
                    duration,
                } => {
                    self.require_subject_combat(&item, subject, requirement, duration);
                }
                Instruction::RequireAllAttacksUntilNextTurn => {
                    self.require_all_attacks_until_next_turn(item.controller);
                }
                Instruction::ProhibitAttacksOnControllerOrPlaneswalkersThisTurn(players) => {
                    self.prohibit_attacks_on_controller_or_planeswalkers_this_turn(&item, players);
                }
                Instruction::RestrictBlockerPower { group, minimum } => {
                    self.restrict_blocker_power(&item, group, minimum);
                }
                Instruction::RequireTargetsBlocked { group } => {
                    self.require_targets_blocked(&item, group);
                }
                Instruction::RequireSubjectBlocked { subject } => {
                    self.require_subject_blocked(&item, subject);
                }
                Instruction::ExileDamagedIfDiesThisTurn
                | Instruction::EquipDiscardAlternative
                | Instruction::ActivationX
                | Instruction::ActivationXWithMinimum { .. }
                | Instruction::ActivationCannotBeCopied
                | Instruction::ActivationNonmanaX
                | Instruction::ActivationLifeX
                | Instruction::TargetsExactlyX
                | Instruction::SourceCounterCost { .. }
                | Instruction::WaterbendCost(_)
                | Instruction::WaterbendXCost { .. }
                | Instruction::TapCreaturesForActivationMana
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
                            maximum: Some(1),
                            selected: Counters::default(),
                        }));
                        self.state.pending = Pending::TargetCounterRemoval {
                            player: item.controller,
                        };
                        return;
                    }
                }
                Instruction::RemoveUpToTargetCounters(_) | Instruction::RemoveAnyTargetCounters => {
                    let maximum = match instruction {
                        Instruction::RemoveUpToTargetCounters(maximum) => Some(u64::from(maximum)),
                        _ => None,
                    };
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
                        && maximum != Some(0)
                        && (maximum.is_none() || self.counter_amounts(target.card).total() != 0)
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
                Instruction::DestroyEntering => {
                    if let Some(id) = self.entering_subject(&item)
                        && !self.effect_destruction_prevented(id)
                    {
                        self.move_object(id, Zone::Graveyard);
                    }
                }
                Instruction::DestroyEventPermanent => {
                    let recipient = item
                        .damage_received
                        .as_ref()
                        .map(|event| event.recipient)
                        .or_else(|| item.event_object.as_ref().map(|event| event.object));
                    if let Some(object) = recipient
                        && self.object_ref(object.card) == object
                        && !self.effect_destruction_prevented(object.card)
                    {
                        self.move_object(object.card, Zone::Graveyard);
                    }
                }
                Instruction::PutColorCountersAndWin { subtype } => {
                    self.state.resolving.as_mut().unwrap().color_sequence =
                        Some(Box::new(color_sequences::ColorCounterSequence {
                            player: item.controller,
                            subtype,
                            next: 0,
                            selected: Vec::new(),
                            successful: Vec::new(),
                            placed: false,
                        }));
                }
                Instruction::PutEventCountersOnTargets | Instruction::PutEventCounters { .. } => {
                    let subject = match instruction {
                        Instruction::PutEventCounters { subject } => subject,
                        _ => Subject::Targets,
                    };
                    let event = item.event_object.as_ref().expect("counter event facts");
                    let EventObjectFacts::DeathCounters(counters) = &event.facts else {
                        unreachable!("counter event facts")
                    };
                    let changes = self
                        .keyword_subjects(&item, subject)
                        .into_iter()
                        .map(|id| (self.object_ref(id), counters.clone()))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
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
                Instruction::CreateTokensThenCounters {
                    token,
                    amount,
                    kind,
                    counters,
                } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::PostEntryCounters {
                            kind,
                            amount: counters,
                        },
                    );
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
                Instruction::CreaturePowerDamageBatch { source, recipients } => {
                    self.creature_power_damage_batch(&item, source, recipients);
                }
                Instruction::PermanentsDealPowerDamage {
                    filters,
                    exclude_source,
                } => self.permanents_deal_damage(&item, filters, exclude_source, None),
                Instruction::PermanentsDealFixedDamage {
                    filters,
                    exclude_source,
                    amount,
                } => self.permanents_deal_damage(&item, filters, exclude_source, Some(amount)),
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
                Instruction::FightForExcess { source, recipient } => {
                    self.fight_for_excess(&item, source, recipient);
                }
                Instruction::GroupPowerDamage { source, recipient } => {
                    self.group_power_damage(&item, std::slice::from_ref(&source), recipient);
                }
                Instruction::GroupsPowerDamage { sources, recipient } => {
                    self.group_power_damage(&item, sources, recipient);
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
                Instruction::AddColorlessScaled(amount)
                | Instruction::AddColorlessScaledFor(_, amount) => {
                    let players = match instruction {
                        Instruction::AddColorlessScaledFor(players, _) => players,
                        _ => PlayerSet::Controller,
                    };
                    let amount = u32::try_from(self.item_amount(amount, &item))
                        .capacity("mana representation exhausted");
                    let mana = [0, 0, 0, 0, 0, amount];
                    let quality = self.item_mana_quality(&item, ManaProduction::Fixed(mana));
                    for player in self.program_players(players, &item) {
                        if quality.is_some() {
                            self.add_quality_mana(player, mana, quality);
                        } else {
                            self.add_mana(player, mana);
                        }
                    }
                }
                Instruction::AddManaSplitScaled {
                    first,
                    second,
                    amount,
                } => {
                    let total = u64::try_from(self.item_amount(amount, &item))
                        .capacity("split mana quantity exhausted");
                    if total != 0 {
                        let production = ManaProduction::Split(crate::cards::ManaSplit {
                            first,
                            second,
                            total,
                        });
                        self.require_mana_split_capacity(item.controller, production);
                        self.state.pending = Pending::EffectMana {
                            player: item.controller,
                            production,
                            recipients: std::array::from_fn(|i| i == item.controller.index()),
                        };
                        return;
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
                Instruction::IfSacrificedVehicle { skip }
                | Instruction::IfSacrificedOutlaw { skip }
                | Instruction::IfSacrificedSuspected { skip } => {
                    let matches =
                        item.paid_additional_cost
                            .as_ref()
                            .is_some_and(|paid| match instruction {
                                Instruction::IfSacrificedVehicle { .. } => paid.sacrificed_vehicle,
                                Instruction::IfSacrificedOutlaw { .. } => paid.sacrificed_outlaw,
                                Instruction::IfSacrificedSuspected { .. } => {
                                    paid.sacrificed_suspected
                                }
                                _ => unreachable!("sacrifice condition"),
                            });
                    if !matches {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::IfPaidDiscardWasNonland { skip } => {
                    let nonland = item
                        .paid_additional_cost
                        .as_ref()
                        .and_then(|paid| paid.discarded_kinds)
                        .is_some_and(|kinds| kinds & 2 == 0);
                    if !nonland {
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
                        | ManaProduction::ChosenColorAmount(_)
                        | ManaProduction::ColorsAmongControlledPermanents
                        | ManaProduction::AnyColorChangingSource(_)
                        | ManaProduction::ChoiceStat { .. }
                        | ManaProduction::MonocoloredPermanentColors
                        | ManaProduction::CraftMaterialColors
                        | ManaProduction::LinkedExileCardColor
                        | ManaProduction::DistinctCreaturePowers
                        | ManaProduction::CountSubtype { .. }
                        | ManaProduction::SourcePower { .. }
                        | ManaProduction::SourcePowerSplit { .. }
                        | ManaProduction::TapSourcePowerChoice { .. }
                        | ManaProduction::TapGreatestControlledCreaturePower { .. }
                        | ManaProduction::TapManaAtCreaturePowerThreshold { .. }
                        | ManaProduction::Tagged { .. } => unreachable!("resolved production"),
                        ManaProduction::Fixed(mana) => {
                            self.add_quality_mana(item.controller, mana, quality)
                        }
                        production @ (ManaProduction::Choice { .. }
                        | ManaProduction::Combination { .. }
                        | ManaProduction::Split(_)) => {
                            self.require_mana_split_capacity(item.controller, production);
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
                    self.event_object_damage_players(&item, players, i128::from(amount));
                }
                Instruction::EventObjectDamageReceivedPlayers(players) => {
                    let amount =
                        i128::try_from(self.item_amount(AmountValue::DamageReceivedAmount, &item))
                            .capacity("damage received representation exhausted");
                    self.event_object_damage_players(&item, players, amount);
                }
                Instruction::DamageEventObjectController(amount) => {
                    self.damage_event_object_controller(&item, amount);
                }
                Instruction::DamageToEventObject(amount) => {
                    self.damage_to_event_object(&item, amount);
                }
                Instruction::SetColors {
                    subject,
                    colors,
                    duration,
                } => {
                    let colors = match colors {
                        crate::cards::ColorValue::Fixed(mask) => mask,
                        crate::cards::ColorValue::Chosen => {
                            1 << self
                                .resolution_chosen_color()
                                .expect("preceding color choice")
                                .index()
                        }
                    };
                    let subjects = self.keyword_subjects(&item, subject);
                    self.set_color_effects(&subjects, colors, duration);
                }
                Instruction::AnimateSource(spec) => self.animate_source(&item, spec),
                Instruction::BecomeLegendaryEquipment {
                    name,
                    equip,
                    power,
                    toughness,
                } => self.become_legendary_equipment(&item, name, equip, power, toughness),
                Instruction::AnimateSourceColored(spec, colors) => {
                    let _ = self.animate_source_colored(&item, spec, colors);
                }
                Instruction::AnimateSourceColoredWithAbility { spec, colors, kind } => {
                    if let Some(form) = self.animate_source_colored(&item, *spec, colors) {
                        self.grant_recipient_ability(
                            form.object,
                            item.trigger_definition(),
                            kind,
                            form.timestamp,
                            form.expires_turn,
                        );
                    }
                }
                Instruction::AnimateSourceWithAbilityIfNoncreature { spec, kind } => {
                    self.animate_source_with_ability_if_noncreature(&item, spec, kind)
                }
                Instruction::AnimateTargets(spec) => self.change_targets_form(
                    &item,
                    spec,
                    characteristic_layers::AnimationTypeMode::AddCreature,
                ),
                Instruction::AnimateTargetsWhileSourceTapped {
                    creature_types,
                    colors,
                    base,
                    keywords,
                } => self.animate_targets_while_source_tapped(
                    &item,
                    creature_types,
                    colors,
                    base,
                    keywords,
                ),
                Instruction::ChangeTargetCreatureForm(spec) => self.change_targets_form(
                    &item,
                    spec,
                    characteristic_layers::AnimationTypeMode::PreserveCardTypes,
                ),
                Instruction::ReplaceTargetCreatureForm(spec) => self.change_targets_form(
                    &item,
                    spec,
                    characteristic_layers::AnimationTypeMode::ReplaceWithCreature,
                ),
                Instruction::AnimateSubject { subject, spec } => {
                    self.animate_subject(&item, subject, spec)
                }
                Instruction::AnimateNoncreatureTargets(spec) => {
                    self.animate_noncreature_targets(&item, spec)
                }
                Instruction::AddArtifactToTargets => self.add_artifact_to_targets(&item),
                Instruction::AddCharacteristics { subject, addition } => {
                    self.add_characteristics(&item, subject, addition);
                }
                Instruction::ChangeSourceCreatureForm(spec) => self.change_source_form(
                    &item,
                    spec,
                    characteristic_layers::AnimationTypeMode::PreserveCardTypes,
                ),
                Instruction::BecomeNamedLegendaryCreature {
                    name,
                    base,
                    ability,
                } => {
                    self.become_named_legendary_creature(&item, name, base, ability);
                }
                Instruction::SetCreatureForm {
                    types,
                    colors,
                    base,
                } => self.set_creature_form(&item, types, colors, base),
                Instruction::MoveCountersFromSource { kind, amount } => {
                    self.move_counters_from_source(&item, kind, amount)
                }
                Instruction::IfSourceCanReceiveCounters { skip }
                | Instruction::IfSubjectCanReceiveCounters { skip, .. } => {
                    let subject = match instruction {
                        Instruction::IfSubjectCanReceiveCounters { subject, .. } => subject,
                        _ => Subject::Source,
                    };
                    let payable = self
                        .resolution_counter_subject(&item, subject)
                        .is_some_and(|object| !replacements::counters_prohibited(self, object));
                    if !payable {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::PaySourceCounterPlacement {
                    player,
                    kind,
                    amount,
                }
                | Instruction::PaySubjectCounterPlacement {
                    player,
                    kind,
                    amount,
                    ..
                } => {
                    let subject = match instruction {
                        Instruction::PaySubjectCounterPlacement { subject, .. } => subject,
                        _ => Subject::Source,
                    };
                    if let Some(object) = self.resolution_counter_subject(&item, subject)
                        && let Some(payer) = self.program_players(player, &item).next()
                    {
                        self.pay_resolution_counter_placement(
                            object,
                            payer,
                            kind,
                            u64::from(amount),
                        );
                    }
                }
                Instruction::AnimateSourceIfNoncreature { spec, ability } => {
                    self.animate_source_if_noncreature(&item, spec, ability)
                }
                Instruction::DestroyLandAndSearch
                | Instruction::DestroyLandThenControllerMaySearch => self.destroy_land_and_search(
                    &item,
                    instruction == Instruction::DestroyLandThenControllerMaySearch,
                ),
                Instruction::SearchLibraryAndGraveyardWithXCounters {
                    filter,
                    counter,
                    haste_at,
                } => {
                    self.start_zone_search(&item, filter, counter, haste_at);
                    return;
                }
                Instruction::SearchLibraryAndOrGraveyardToHand(filter)
                | Instruction::SearchRequiredLibraryAndOrGraveyardToHand(filter) => {
                    let may_decline = matches!(
                        instruction,
                        Instruction::SearchLibraryAndOrGraveyardToHand(_)
                    );
                    self.start_zone_hand_search(&item, filter, may_decline);
                    return;
                }
                Instruction::SearchHandAndOrLibrary(filter)
                | Instruction::SearchGraveyardHandAndOrLibrary(filter) => {
                    let include_graveyard =
                        matches!(instruction, Instruction::SearchGraveyardHandAndOrLibrary(_));
                    self.start_hand_library_search(item.controller, filter, include_graveyard);
                    return;
                }
                Instruction::SearchGraveyardHandLibraryForNames(names) => {
                    self.start_named_hand_library_search(item.controller, names);
                    return;
                }
                Instruction::SearchCreatureManaValue {
                    maximum,
                    destination,
                } => {
                    let maximum =
                        u64::try_from(self.item_amount(maximum, &item)).unwrap_or(u64::MAX);
                    self.state.resolving.as_mut().unwrap().search =
                        Some(Box::new(search::SearchChoice {
                            defer_shuffle: false,
                            spec: SearchSpec {
                                filter: crate::SearchFilter::KindsManaValue {
                                    kinds: 1,
                                    minimum: 0,
                                    maximum,
                                },
                                maximum: 1,
                                destination,
                                tapped: false,
                                reveal: true,
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
                    self.state.pending = Pending::EffectSearch {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::SearchTwoQualitiesToHand(roles) => {
                    self.state.resolving.as_mut().unwrap().search =
                        Some(Box::new(search::SearchChoice {
                            spec: SearchSpec {
                                filter: crate::SearchFilter::AnyOf(roles),
                                maximum: 2,
                                destination: crate::SearchDestination::Hand,
                                tapped: false,
                                reveal: true,
                            },
                            defer_shuffle: false,
                            entry_counters: &[],
                            minimum: 0,
                            different_names: false,
                            distinct_colors: 0,
                            paired_qualities: Some(roles),
                            selected: Vec::new(),
                            distributed: None,
                            destination_pending: false,
                        }));
                    self.state.pending = Pending::EffectSearch {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::SearchFor { spec, .. }
                | Instruction::Search(spec)
                | Instruction::SearchWithoutShuffle(spec)
                | Instruction::SearchDistinctColors { spec, .. }
                | Instruction::SearchCreaturesUpToChosenX(spec)
                | Instruction::SearchScaled { spec, .. }
                | Instruction::SearchDamageReceived(spec)
                | Instruction::SearchTargetController(spec)
                | Instruction::SearchTargetControllerWithEntryCounters { spec, .. }
                | Instruction::SearchTargetPlayer(spec)
                | Instruction::SearchTargetPlayerMovedCount(spec)
                | Instruction::SearchRequired(spec)
                | Instruction::SearchDifferentNames(spec)
                | Instruction::SearchDifferentNamesChosenX(spec) => {
                    let mut spec = spec;
                    let distinct_colors =
                        if let Instruction::SearchDistinctColors { colors, .. } = instruction {
                            assert!(colors & !0x1f == 0, "only WUBRG search roles");
                            spec.filter = crate::SearchFilter::Colors(colors);
                            spec.maximum = spec.maximum.min(u64::from(colors.count_ones()));
                            colors
                        } else {
                            0
                        };
                    if matches!(instruction, Instruction::SearchCreaturesUpToChosenX(_)) {
                        assert_eq!(
                            spec.filter,
                            crate::SearchFilter::Creature,
                            "chosen-X creature search requires a creature filter"
                        );
                        spec.filter = crate::SearchFilter::KindsManaValue {
                            kinds: 1,
                            minimum: 0,
                            maximum: u64::from(item.x.unwrap_or(0)),
                        };
                    }
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
                    let player = if let Instruction::SearchFor { player, .. } = instruction {
                        let Some(player) = self.program_players(player, &item).next() else {
                            continue;
                        };
                        player
                    } else if matches!(
                        instruction,
                        Instruction::SearchTargetController(_)
                            | Instruction::SearchTargetControllerWithEntryCounters { .. }
                    ) {
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
                    let minimum = if matches!(
                        instruction,
                        Instruction::SearchRequired(_)
                            | Instruction::SearchFor { required: true, .. }
                    ) {
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
                            defer_shuffle: matches!(
                                instruction,
                                Instruction::SearchWithoutShuffle(_)
                            ),
                            spec,
                            entry_counters: match instruction {
                                Instruction::SearchTargetControllerWithEntryCounters {
                                    amounts,
                                    ..
                                } => amounts,
                                _ => &[],
                            },
                            minimum,
                            different_names: matches!(
                                instruction,
                                Instruction::SearchDifferentNames(_)
                                    | Instruction::SearchDifferentNamesChosenX(_)
                            ),
                            distinct_colors,
                            paired_qualities: None,
                            selected: Vec::new(),
                            distributed: None,
                            destination_pending: false,
                        }));
                    self.state.pending = Pending::EffectSearch { player };
                    return;
                }
                Instruction::ShuffleAfterSearches => {
                    if std::mem::take(
                        &mut self
                            .state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .deferred_search_shuffle,
                    ) {
                        self.forget_library_top(item.controller);
                        self.shuffle_library_cards(item.controller);
                    }
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
                            defer_shuffle: false,
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
                            entry_counters: &[],
                            minimum: 0,
                            different_names: false,
                            distinct_colors: 0,
                            paired_qualities: None,
                            selected: Vec::new(),
                            distributed: None,
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
                Instruction::AddEntryResultCharacteristics(addition) => {
                    if let Some(recipient) = self.captured_entry_result() {
                        self.add_characteristics_to(&[recipient], addition);
                    }
                }
                Instruction::IfEntryResultSubtype { subtypes, skip } => {
                    if !self.captured_entry_result_has_subtype(subtypes) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::CopyNonlegendaryEntryResult { base } => {
                    if let Some(recipient) = self.captured_entry_result()
                        && !self.object_view(recipient.card).is_legendary()
                    {
                        let recipe =
                            self.copiable_recipe(recipient)
                                .with_exceptions(CopyExceptions {
                                    base: Some(base),
                                    ..CopyExceptions::NONE
                                });
                        self.stage_copy_recipes(item.controller, vec![recipe], 1, false);
                    }
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
                Instruction::GrantEntryResultTriggers(abilities) => {
                    if let Some(recipient) = self.captured_entry_result() {
                        let timestamp = self.next_role_timestamp();
                        self.grant_permanent_triggers(&item, recipient, abilities, timestamp);
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
                Instruction::ReturnGraveyardTargetsWithHasteAndDelayedSacrifice => {
                    self.return_targets_with_haste_and_delayed_sacrifice(&item);
                }
                Instruction::ReturnGraveyardEquipmentAttachedToSource => {
                    self.return_equipment_targets_attached_to_source(&item);
                }
                Instruction::AnimatePermanentGroup { filters, spec } => {
                    self.animate_permanent_group(&item, filters, spec);
                }
                Instruction::CaptureSearchResult => {
                    self.state.resolving.as_mut().unwrap().search_result =
                        Some(Box::new(search::SearchResult::default()));
                }
                Instruction::CastExiledSearchResultFreeUpTo(maximum) => {
                    if self.offer_exiled_search_result_free_cast(item.controller, maximum) {
                        return;
                    }
                }
                Instruction::PutExiledSearchResultInHand => {
                    if let Some(object) = self.exiled_search_result() {
                        self.move_object(object.card, Zone::Hand);
                    }
                }
                Instruction::IfSearchPutInHand { skip } => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    if !resolution
                        .search_result
                        .as_ref()
                        .is_some_and(|result| result.put_in_hand)
                    {
                        resolution.pc += skip;
                    }
                }
                Instruction::BeholdToUntapSearchResult(subtype) => {
                    if self.start_behold_search_result(&item, subtype) {
                        return;
                    }
                }
                Instruction::MayBehold { subtype, .. } => {
                    self.start_optional_behold(&item, subtype);
                    return;
                }
                Instruction::IfBeheld { beheld, skip } => {
                    if item.cast.is_some_and(|cast| cast.beheld == Some(true)) != beheld {
                        self.state.resolving.as_mut().unwrap().pc += skip;
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
                Instruction::RevealTopToHandLoseManaValue => {
                    self.reveal_top_to_hand_lose_mana_value(&item, PlayerSet::Controller)
                }
                Instruction::RevealTopToHandLoseManaValueFor(players) => {
                    self.reveal_top_to_hand_lose_mana_value(&item, players);
                }
                Instruction::RevealTargetTopsToHandLoseOtherManaValue => {
                    self.reveal_target_tops_to_hand_lose_other_mana_value(&item)
                }
                Instruction::ExileSourceReturnTransformed { counter } => {
                    self.exile_source_return_transformed(&item, counter)
                }
                Instruction::ExileBattlefieldSourceReturn {
                    transformed,
                    under_owner,
                } => self.exile_battlefield_source_return(&item, transformed, under_owner),
                Instruction::ExileBattlefieldSourceReturnToOpponent => {
                    self.blink_source_under_opponent(&item);
                }
                Instruction::ReturnGraveyardSourceTransformed => {
                    self.return_graveyard_source_transformed(&item)
                }
                Instruction::ReturnDepartedSourceTransformedTapped { counters } => {
                    self.return_departed_source_transformed_tapped(&item, counters)
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
                Instruction::ExileEachLibraryExceptBottomFaceDown(keep) => {
                    self.exile_each_library_except_bottom_face_down(keep);
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
                Instruction::RevealHandForControlledCast(amount) => {
                    if self.start_hand_subset_controlled_cast(&item, amount) {
                        return;
                    }
                }
                Instruction::MarkCastDreamExile => self.mark_cast_dream_exile(&item),
                Instruction::CastOwnedDreamCard => {
                    self.start_dream_free_cast(item.controller);
                    return;
                }
                Instruction::ExileUntilNonlandCastByDiscarding => {
                    if self.offer_first_exiled_nonland_for_discard(item.controller) {
                        return;
                    }
                }
                Instruction::ExileNonlandLifeFreeCast => {
                    if self.exile_nonland_life_free_cast(item.controller) {
                        return;
                    }
                }
                Instruction::ExileUntilManaValueFreeCast { players, minimum } => {
                    for owner in self.program_players(players, &item) {
                        self.exile_until_mana_value_free_cast(owner, item.controller, minimum);
                    }
                }
                Instruction::ExileTopChoosePlayScaled { amount, next_turn } => {
                    let count = self.item_amount(amount, &item);
                    if self.exile_top_choose_play(item.controller, count, next_turn) {
                        return;
                    }
                }
                Instruction::ExileDeathCounterTotalUntilNextEnd => {
                    let amount = self.item_amount(AmountValue::DeathEventCounterTotal, &item);
                    self.exile_top_play_until_next_end_step(item.controller, amount);
                }
                Instruction::ExileTopChoosePlay { amount, next_turn } => {
                    if self.exile_top_choose_play(item.controller, u128::from(amount), next_turn) {
                        return;
                    }
                }
                Instruction::RandomLinkedExileToBattlefield => {
                    self.random_linked_exile_to_battlefield(&item)
                }
                Instruction::ExileTopPlayScaled { amount, next_turn } => {
                    let count = self.item_amount(amount, &item);
                    self.exile_top_play(item.controller, count, next_turn);
                }
                Instruction::ExileTopPlayForPlayer {
                    player,
                    amount,
                    next_turn,
                } => {
                    if let Some(player) = self.program_players(player, &item).next() {
                        let count = self.item_amount(amount, &item);
                        self.exile_top_play(player, count, next_turn);
                    }
                }
                Instruction::ExileTopLibrary { players, amount }
                | Instruction::ExileTopLibraryRemember { players, amount } => {
                    let count = self.item_amount(amount, &item);
                    let ids = self
                        .program_players(players, &item)
                        .flat_map(|player| exile::top_library_cards(self, player, count))
                        .collect::<Vec<_>>();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    if matches!(instruction, Instruction::ExileTopLibraryRemember { .. }) {
                        let resolution = self.state.resolving.as_mut().unwrap();
                        let actual = events
                            .iter()
                            .filter(|event| event.moved && event.destination == Zone::Exile);
                        resolution.exiled_library_land =
                            actual.clone().any(|event| event.before.kinds & 2 != 0);
                        resolution.exiled_library_nonland =
                            actual.clone().any(|event| event.before.kinds & 2 == 0);
                        resolution.milled = actual.map(|event| event.after).collect();
                    }
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::IfExiledLibraryCard { land, skip } => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    let matched = if land {
                        resolution.exiled_library_land
                    } else {
                        resolution.exiled_library_nonland
                    };
                    if !matched {
                        resolution.pc += skip;
                    }
                }
                Instruction::ExileEachTopPlayOwnAndLesserFree => {
                    self.exile_each_top_play_own_and_lesser_free(item.controller);
                }
                Instruction::ExileTopPlayUntilNextEndStep(amount) => {
                    self.exile_top_play_until_next_end_step(item.controller, u128::from(amount));
                }
                Instruction::ExileTopPlayOnSourceLoreTurns => {
                    self.exile_top_play_on_source_lore_turns(&item);
                }
                Instruction::ExileTopPlayUntilSourceExilesAnother => {
                    self.exile_top_play_until_source_exiles_another(&item);
                }
                Instruction::ExileTopPlayWhileSourceControlled => {
                    self.exile_top_play_while_source_controlled(&item);
                }
                Instruction::ExileTopPlayWhileControlSubtype(subtype) => {
                    self.exile_top_play_while_control_subtype(item.controller, subtype);
                }
                Instruction::ExileUntilNonlandCastThisTurn => {
                    self.exile_until_nonland_cast_this_turn(item.controller);
                }
                Instruction::ExileTopPlay { amount, next_turn } => {
                    self.exile_top_play(item.controller, u128::from(amount), next_turn);
                }
                Instruction::GrantLibraryTopPlayUntilEndTurn => {
                    self.state.players[item.controller.index()].library_top_play = true;
                }
                Instruction::GrantLibraryTopLifePlayUntilEndTurn => {
                    self.state.players[item.controller.index()].library_top_life_play = true;
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
                    let is_odd = !self.item_amount(value, &item).is_multiple_of(2);
                    if is_odd != odd {
                        let resolution = self.state.resolving.as_mut().unwrap();
                        let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                        assert!(skip_to <= resolution.program.len());
                        resolution.pc = skip_to;
                    }
                }
                Instruction::RememberCount(value) => {
                    let count = u64::try_from(self.item_amount(value, &item))
                        .capacity("saved resolution count exhausted");
                    self.state.resolving.as_mut().unwrap().moved_count = Some(count);
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
                Instruction::PlayHideawayIfCoven | Instruction::PlayHideaway => {
                    let allowed =
                        instruction == Instruction::PlayHideaway || self.has_coven(item.controller);
                    if allowed && self.play_hideaway(&item) {
                        return;
                    }
                }
                Instruction::SelectLibraryThenMayTransform(effect) => {
                    if self.start_library_selection_completion(
                        &item,
                        effect,
                        library_selection::LibrarySelectionCompletion::MayTransform,
                    ) {
                        return;
                    }
                }
                Instruction::SelectLibraryToExileRemember(effect) => {
                    if self.start_library_selection_completion(
                        &item,
                        effect,
                        library_selection::LibrarySelectionCompletion::RememberExile,
                    ) {
                        return;
                    }
                }
                Instruction::CastRememberedExileFree => {
                    let exiled = std::mem::take(&mut self.state.resolving.as_mut().unwrap().milled);
                    if let Some(&object) = exiled.first()
                        && self.object_ref(object.card) == object
                        && self.state.objects[object.card as usize].zone == Zone::Exile
                    {
                        self.offer_exact_free_cast(item.controller, object, Zone::Exile, None);
                        return;
                    }
                }
                Instruction::CastOneRememberedExileFreeThenRestToHand { minimum } => {
                    if self
                        .cast_one_remembered_exile_free_then_rest_to_hand(item.controller, minimum)
                    {
                        return;
                    }
                }
                Instruction::SelectLibraryThenReflexiveEquipment { effect, ability } => {
                    if self.start_library_selection_completion(
                        &item,
                        effect,
                        library_selection::LibrarySelectionCompletion::ReflexiveEquipment(ability),
                    ) {
                        return;
                    }
                }
                Instruction::AlternateRevealedLibraryPicks(look) => {
                    if self.start_alternating_library_picks(&item, look) {
                        return;
                    }
                }
                Instruction::SelectTargetLibraryFaceDownPlay {
                    look,
                    count,
                    spend_any_mana,
                } => {
                    if self.select_target_library_face_down_play(&item, look, count, spend_any_mana)
                    {
                        return;
                    }
                }
                Instruction::SelectLibraryTo(effect) => {
                    if self.start_library_selection_to(&item, effect) {
                        return;
                    }
                }
                Instruction::SelectLibraryToHandRememberCount(effect) => {
                    if self.start_library_selection_completion(
                        &item,
                        effect,
                        library_selection::LibrarySelectionCompletion::RememberHandCount,
                    ) {
                        return;
                    }
                }
                Instruction::SelectLibrary(spec) => {
                    if self.start_library_selection(&item, spec) {
                        return;
                    }
                }
                Instruction::ScryTargetPlayer(amount) => {
                    if let Some(player) = self.program_players(PlayerSet::Targeted, &item).next()
                        && self.start_library_arrangement(player, usize::from(amount), false)
                    {
                        return;
                    }
                }
                Instruction::Scry(_) | Instruction::Surveil(_) | Instruction::SurveilScaled(_) => {
                    let amount = match instruction {
                        Instruction::Scry(amount) | Instruction::Surveil(amount) => {
                            usize::from(amount)
                        }
                        Instruction::SurveilScaled(value) => {
                            usize::try_from(self.item_amount(value, &item)).unwrap_or(usize::MAX)
                        }
                        _ => unreachable!("scry or surveil amount"),
                    };
                    if self.start_library_arrangement(
                        item.controller,
                        amount,
                        !matches!(instruction, Instruction::Scry(_)),
                    ) {
                        return;
                    }
                }
                Instruction::SurveilRememberTop(amount) => {
                    self.state.resolving.as_mut().unwrap().moved_count = Some(0);
                    if self.start_library_arrangement(item.controller, usize::from(amount), true) {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .library
                            .as_mut()
                            .unwrap()
                            .remember_top = true;
                        return;
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
                        self.state.resolving.as_mut().unwrap().library =
                            Some(Box::new(LibraryChoice {
                                surveil: true,
                                remember_top: false,
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
                            }));
                        self.state.pending = Pending::EffectLibrary {
                            player: item.controller,
                        };
                        return;
                    }
                    self.scry_or_surveil_event(item.controller);
                }
                Instruction::CounterRelatedUnlessSacrifice(_)
                | Instruction::CounterRelatedUnlessSacrificeSubtype(_) => {
                    let (amount, subtype) = match instruction {
                        Instruction::CounterRelatedUnlessSacrifice(amount) => (amount, None),
                        Instruction::CounterRelatedUnlessSacrificeSubtype(subtype) => {
                            (1, Some(subtype))
                        }
                        _ => unreachable!(),
                    };
                    let related = item.related_stack.expect("ward payer and related object");
                    let mut payer_item = item.clone();
                    payer_item.controller = related.controller;
                    let pending = self.start_effect_choice(
                        &payer_item,
                        EffectChoiceKind::Sacrifice,
                        PlayerSet::Controller,
                        SacrificeFilter {
                            subtype,
                            ..evidence::cost(0).filter
                        },
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
                Instruction::CounterRelatedUnlessDiscardRandom => {
                    if self.start_ward_random_discard(&item) {
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
                            .life = Some(i128::from(life));
                        return;
                    }
                }
                Instruction::PayXReflexive { .. } | Instruction::PayXReflexiveLimited { .. } => {
                    self.state.pending = Pending::ResolutionX {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::MayPayCosts { costs, skip } => {
                    self.start_resolution_costs(&item, item.controller, None, costs, false, skip);
                    return;
                }
                Instruction::MayPayCostsFor {
                    players,
                    costs,
                    skip,
                } => {
                    if let Some(payer) = self.program_players(players, &item).next() {
                        self.start_resolution_costs(&item, payer, None, costs, false, skip);
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc += skip;
                }
                Instruction::MayPayMana { cost, skip } => {
                    self.start_optional_resolution_payment(item.controller, cost, skip);
                    return;
                }
                Instruction::MayPayManaAlternatives { costs, skip } => {
                    let (&cost, alternatives) =
                        costs.split_first().expect("nonempty payment choices");
                    self.start_optional_resolution_payment(item.controller, cost, skip);
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
                Instruction::CreateSizedToken { token, size } => {
                    let size = self.item_amount(size, &item);
                    self.create_sized_token(item.controller, token, size);
                }
                Instruction::CreateSizedTokenFromDeathBatchPower { token } => {
                    let Some(EventObjectFacts::DeathBatchPower(power)) =
                        item.event_object.as_ref().map(|event| &event.facts)
                    else {
                        unreachable!("sized token requires a death batch power receipt");
                    };
                    self.create_token_with_sampled_base(
                        item.controller,
                        token,
                        copies::SampledTokenSize::signed(*power),
                    );
                }
                Instruction::EndureScaled {
                    recipient,
                    amount,
                    token,
                } => {
                    let amount = self.item_amount(amount, &item);
                    if self.start_scaled_endure(&item, recipient, amount, token) {
                        return;
                    }
                }
                Instruction::Endure { amount, token } => {
                    if self.start_fixed_endure(&item, amount, token) {
                        return;
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
                Instruction::RemoveAllSourceCounters(kind) => {
                    if let Some(source) = item.source
                        && self.on_battlefield(source.id)
                        && let Some(amount) = self.source_counter_quantity_live(source.object, kind)
                    {
                        self.pay_counters(source.id, &Counters::of(kind, amount));
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
                | Instruction::ChooseCreatureType
                | Instruction::ChooseSecretCreatureType(_)
                | Instruction::CreateEmblemForChosenCreatureType(_)
                | Instruction::DestroyCreaturesExceptChosenType => {
                    self.state.pending = Pending::CreatureTypeChoice {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::IfTargetHasRevealedCreatureType { skip } => {
                    let matches = item.event_object.as_ref().is_some_and(|event| {
                        let EventObjectFacts::RevealedCreatureType(kind) = event.facts else {
                            return false;
                        };
                        item.targets.iter().any(|target| {
                            self.same_target_object(*target)
                                && target.object.is_some_and(|object| {
                                    self.has_subtype(object.card, kind.name())
                                })
                        })
                    });
                    if !matches {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::CreateTokensWithSourceDeparture {
                    token,
                    amount,
                    ability,
                } => {
                    let amount = self.item_amount(amount, &item);
                    if amount == 0 {
                        self.register_created_token_departure(&item, ability, Vec::new());
                    } else {
                        self.stage_token_creation(
                            item.controller,
                            token,
                            amount,
                            false,
                            replacements::TokenCompletion::SourceDeparture {
                                item: Box::new(item),
                                ability,
                            },
                        );
                    }
                }
                Instruction::ExileDelayedCreatedTokens => self.exile_delayed_created_tokens(&item),
                Instruction::ExileDelayedBattlefieldObjects => {
                    self.exile_delayed_battlefield_objects(&item)
                }
                Instruction::CreateTokensScaled {
                    token,
                    amount,
                    tapped,
                } => {
                    let amount = self.item_amount(amount, &item);
                    self.create_tokens(item.controller, token, amount, tapped);
                }
                Instruction::CreateTokensRepeated { token, amount } => {
                    let remaining = self
                        .state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .repeated_instruction_remaining
                        .take()
                        .unwrap_or_else(|| {
                            u32::try_from(self.item_amount(amount, &item))
                                .capacity("token event count exhausted")
                        });
                    if remaining > 0 {
                        if remaining > 1 {
                            let resolution = self.state.resolving.as_mut().unwrap();
                            resolution.repeated_instruction_remaining = Some(remaining - 1);
                            resolution.pc -= 1;
                        }
                        self.create_tokens(item.controller, token, 1, false);
                    }
                }
                Instruction::PutLandCards { maximum, tapped } => {
                    let maximum = u32::try_from(self.item_amount(maximum, &item))
                        .capacity("land selection limit exhausted");
                    if self.start_land_selection(&item, maximum, tapped) {
                        return;
                    }
                }
                Instruction::PutHandLands { maximum, tapped } => {
                    let maximum =
                        u32::try_from(self.item_amount(maximum, &item)).unwrap_or(u32::MAX);
                    if self.start_land_selection(&item, maximum, tapped) {
                        return;
                    }
                }
                Instruction::PutHandCreatures { maximum } => {
                    if self.start_hand_creatures(&item, maximum) {
                        return;
                    }
                }
                Instruction::PutHandCreatureHasteSacrifice {
                    maximum_total_stats,
                } => {
                    if self.start_hand_creature_haste_sacrifice(&item, maximum_total_stats) {
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
                Instruction::PutHandCreatureAndLand => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::HandCreatureAndLand,
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        2,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::PutHandCreature => {
                    if self.start_hand_creature(&item) {
                        return;
                    }
                }
                Instruction::PutHandCreatureAttacking { maximum } => {
                    if self.start_hand_creature_attacking(&item, maximum) {
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
                Instruction::ExileOpposingCreaturesThatWouldDieThisTurn => {
                    self.state.players[item.controller.index()].exile_opposing_creature_deaths =
                        true;
                }
                Instruction::CreateRoleReflexive { token, ability } => {
                    self.resolve_role_reflexive(&item, token, ability);
                }
                Instruction::EventObjectPowerDamage => self.event_object_power_damage(&item),
                Instruction::AttachedCreaturePowerDamage => {
                    self.attached_creature_power_damage(&item)
                }
                Instruction::ReflexiveFight => {
                    self.reflexive_fight(&item);
                }
                Instruction::CreateRolesMatching { token, filter } => {
                    self.resolve_roles_matching(&item, token, filter);
                }
                Instruction::CreateRole { token, subject } => {
                    self.resolve_role(&item, token, subject);
                }
                Instruction::CreateTokensThenMayAttachSource { token, amount } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::OptionalAttachSource(
                            item.source
                                .expect("attachment requires an object source")
                                .object,
                        ),
                    );
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
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    if let [player] = players.as_slice() {
                        self.create_tokens(*player, token, u128::from(amount), tapped);
                    } else {
                        self.stage_simultaneous_token_creation(
                            &players,
                            TokenGroupView {
                                token,
                                amount: u128::from(amount),
                                tapped,
                            },
                            None,
                        );
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
                Instruction::CreateTokensThenSuspect { token, amount } => {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        u128::from(amount),
                        false,
                        replacements::TokenCompletion::Suspect,
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
                Instruction::CreateTokenBatch(groups) => {
                    self.stage_fixed_token_batch(item.controller, groups);
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
                Instruction::IfOptionalAbilityUnused { skip }
                | Instruction::MayOncePerTurn { skip }
                | Instruction::MayCopyOncePerTurn { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    let unavailable = matches!(instruction, Instruction::MayCopyOncePerTurn { .. })
                        && self.captured_copy_template(&item).is_none();
                    if unavailable || self.optional_ability_used(&item) {
                        self.state.resolving.as_mut().unwrap().pc = skip_to;
                    } else if !matches!(instruction, Instruction::IfOptionalAbilityUnused { .. }) {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                }
                Instruction::MayForPlayer { player, skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if let Some(player) = self.program_players(player, &item).next() {
                        self.state.pending = Pending::EffectOptional { player, skip_to };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::May { skip }
                | Instruction::MayIfTargetGroupsPresent { skip, .. }
                | Instruction::MayIfSourceOnBattlefield { skip }
                | Instruction::MayTransformSource { skip }
                | Instruction::MayIfSourceInZone { skip, .. }
                | Instruction::MayIfEventPermanentOnBattlefield { skip }
                | Instruction::MayIfCombatDamageDealerOnBattlefield { skip }
                | Instruction::MayIfRememberedExiledCreature { skip }
                | Instruction::MayDraw { skip }
                | Instruction::MayCopyRelatedTriggeredAbility { skip }
                | Instruction::MayExileTop { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    let available = match instruction {
                        Instruction::MayIfTargetGroupsPresent { groups, .. } => groups
                            .iter()
                            .all(|&group| item.targets.iter().any(|bound| bound.group == group)),
                        Instruction::MayIfSourceOnBattlefield { .. } => self
                            .exact_source_in_zone(&item, TargetZone::Battlefield)
                            .is_some(),
                        Instruction::MayTransformSource { .. } => {
                            self.transformable_source(&item).is_some()
                        }
                        Instruction::MayIfSourceInZone { zone, .. } => {
                            self.exact_source_in_zone(&item, zone).is_some()
                        }
                        Instruction::MayIfEventPermanentOnBattlefield { .. } => {
                            self.entering_subject(&item).is_some()
                        }
                        Instruction::MayIfCombatDamageDealerOnBattlefield { .. } => {
                            self.live_combat_damage_dealer(&item).is_some()
                        }
                        Instruction::MayIfRememberedExiledCreature { .. } => {
                            self.remembered_exiled_creature_copy(&item).is_some()
                        }
                        Instruction::MayDraw { .. } => !self.draws_prohibited(),
                        Instruction::MayCopyRelatedTriggeredAbility { .. } => {
                            self.related_triggered_ability(&item).is_some()
                        }
                        Instruction::MayExileTop { .. } => {
                            self.top_library_card(item.controller).is_some()
                        }
                        _ => true,
                    };
                    if !available {
                        self.state.resolving.as_mut().unwrap().pc = skip_to;
                        continue;
                    }
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
                    self.airbend_other_creatures(item.controller, exception);
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
                    let expiry = if next_turn {
                        ExilePlayExpiry::EndOfNextTurn
                    } else {
                        ExilePlayExpiry::Turn(self.state.turn)
                    };
                    self.exile_targets_play(&item, expiry, false, false)
                }
                Instruction::ExileTargetsAndTopPlayForOwners => {
                    self.exile_targets_play(&item, ExilePlayExpiry::EndOfNextTurn, true, true)
                }
                Instruction::ExileTargetsPlayForOwnersWhileExiled => {
                    self.exile_targets_play(&item, ExilePlayExpiry::WhileExiled, true, false)
                }
                Instruction::ExileDepartedSource => {
                    self.exile_departed_source(&item);
                }
                Instruction::MayExileDepartedSource { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    if self.departed_source_in_graveyard(&item).next().is_some() {
                        self.state.pending = Pending::EffectOptional {
                            player: item.controller,
                            skip_to,
                        };
                        return;
                    }
                    self.state.resolving.as_mut().unwrap().pc = skip_to;
                }
                Instruction::ExileDepartedSourceReflexive { ability } => {
                    if self.exile_departed_source(&item) {
                        self.queue_reflexive(&item, ability, None);
                    }
                }
                Instruction::ExileUntilSourceLeaves
                | Instruction::ExileUntilSourceLeavesRememberPowerAtMost(_) => {
                    let events = self.exile_until_source_leaves(&item);
                    if let Instruction::ExileUntilSourceLeavesRememberPowerAtMost(maximum) =
                        instruction
                    {
                        let count = events
                            .iter()
                            .filter(|event| {
                                event.moved
                                    && event.destination == Zone::Exile
                                    && event.before.zone == Zone::Battlefield
                                    && event.before.kinds & 1 != 0
                                    && event.before.power <= i128::from(maximum)
                            })
                            .count();
                        self.state.resolving.as_mut().unwrap().moved_count =
                            Some(count.try_into().capacity("exiled creature count exhausted"));
                    }
                }
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
                Instruction::AddCastEntryCountersForControlledSubtypes { kind, subtypes } => {
                    self.add_cast_entry_counters_for_subtypes(&item, kind, subtypes)
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
                    self.draw_then_discard_if_drew(
                        item.controller,
                        self.item_amount(amount, &item),
                        item.id,
                        discard,
                    );
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
                Instruction::DrawToHandSize { players, size } => {
                    for player in self.program_players(players, &item) {
                        let count = usize::from(size).saturating_sub(self.hand_count(player));
                        self.draw_many(player, count as u128);
                    }
                }
                Instruction::DrawAndLoseLife(value) => {
                    let amount = self.item_amount(value, &item);
                    self.draw_then_lose_life(item.controller, amount);
                }
                Instruction::DrawRemembered(value) => {
                    let amount = self.item_amount(value, &item);
                    self.state.resolving.as_mut().unwrap().remembered_draws = Some(0);
                    self.draw_many_recording(item.controller, amount, item.id);
                }
                Instruction::IfNoRememberedDraws { skip } => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    if resolution.remembered_draws.take().unwrap_or(0) > 0 {
                        resolution.pc += skip;
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
                Instruction::OpponentSacrificeDiscardOrLoseLife { repetitions, life } => {
                    let count = self.item_amount(repetitions, &item);
                    self.start_repeated_life_choice(count, life);
                }
                Instruction::MayDiscardOrLoseLife { players, amount } => {
                    let mut affected = [false; 2];
                    for player in self.program_players(players, &item) {
                        affected[player.index()] = true;
                    }
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards = affected.map(usize::from);
                    resolution.discard_after =
                        Some(discard_events::DiscardAfter::LoseLifeForDeclinedDiscard {
                            players: affected,
                            paid: [false; 2],
                            amount,
                        });
                }
                Instruction::DiscardRandom(set, amount) => {
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    let mut selected = Vec::new();
                    for player in players {
                        self.select_random_discards(player, amount, &mut selected);
                    }
                    self.discard_batch_caused(&selected, None, Some(item.controller));
                }
                Instruction::DiscardOneRememberCard => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards[item.controller.index()] = 1;
                    resolution.discard_after =
                        Some(discard_events::DiscardAfter::RememberOne(item.controller));
                }
                Instruction::DiscardOneReflexiveManaValue(ability)
                | Instruction::DiscardExactlyReflexiveManaValue { ability, .. } => {
                    let amount = match instruction {
                        Instruction::DiscardExactlyReflexiveManaValue { amount, .. } => amount,
                        _ => 1,
                    };
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards[item.controller.index()] = usize::from(amount);
                    resolution.discard_after =
                        Some(discard_events::DiscardAfter::ReflexiveManaValue {
                            controller: item.controller,
                            amount,
                            ability,
                        });
                }
                Instruction::CastRememberedDiscardFree => {
                    let card = self.state.resolving.as_mut().unwrap().discarded_card.take();
                    if let Some(card) = card
                        && self.offer_discard_result_free_cast(item.controller, card)
                    {
                        return;
                    }
                }
                Instruction::DiscardTwoNonlandReflexive(ability) => {
                    self.state.resolving.as_mut().unwrap().discards[item.controller.index()] = 2;
                    self.state.resolving.as_mut().unwrap().discard_after =
                        Some(discard_events::DiscardAfter::ReflexiveCount {
                            controller: item.controller,
                            ability,
                            nonland_only: true,
                        });
                }
                Instruction::SearchLibraryOrOutsideLegendarySplit {
                    maximum,
                    opponent_choices,
                } => {
                    self.start_legendary_split_search(item.controller, maximum, opponent_choices);
                    return;
                }
                Instruction::ChooseOutsideGameCard => {
                    self.start_single_outside_choice(item.controller, item.id);
                    return;
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
                Instruction::DiscardOneEachDrawUnlessManaValue { players, minimum } => {
                    let mut affected = [false; 2];
                    for player in self.program_players(players, &item) {
                        affected[player.index()] = true;
                    }
                    let resolution = self.state.resolving.as_mut().unwrap();
                    resolution.discards = affected.map(usize::from);
                    resolution.discard_after =
                        Some(discard_events::DiscardAfter::DrawUnlessDiscardedManaValue {
                            controller: item.controller,
                            affected,
                            minimum,
                        });
                }
                Instruction::DiscardThenDrawForTypes { players, amount }
                | Instruction::DiscardThenDrawForOwnCount { players, amount } => {
                    for player in self.program_players(players, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] =
                            usize::from(amount);
                    }
                    self.state.resolving.as_mut().unwrap().discard_after =
                        Some(match instruction {
                            Instruction::DiscardThenDrawForOwnCount { .. } => {
                                discard_events::DiscardAfter::DrawActual {
                                    players: std::array::from_fn(|index| {
                                        index == item.controller.index()
                                    }),
                                    bonus: 0,
                                }
                            }
                            _ => discard_events::DiscardAfter::DrawCardTypes(item.controller),
                        });
                }
                Instruction::DiscardOneMatchingOr { filter, otherwise } => {
                    if self.start_effect_choice(
                        &item,
                        EffectChoiceKind::DiscardOneMatchingOr { filter, otherwise },
                        PlayerSet::Controller,
                        evidence::cost(0).filter,
                        1,
                        Some(0),
                    ) {
                        return;
                    }
                }
                Instruction::Discard(set, _)
                | Instruction::DiscardScaled(set, _)
                | Instruction::DiscardRememberCount { players: set, .. } => {
                    let amount = match instruction {
                        Instruction::Discard(_, n)
                        | Instruction::DiscardRememberCount { amount: n, .. } => usize::from(n),
                        Instruction::DiscardScaled(_, value) => {
                            usize::try_from(self.item_amount(value, &item))
                                .capacity("discard amount exhausted")
                        }
                        _ => unreachable!(),
                    };
                    for player in self.program_players(set, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] = amount;
                    }
                    if matches!(instruction, Instruction::DiscardRememberCount { .. }) {
                        let resolution = self.state.resolving.as_mut().unwrap();
                        resolution.moved_count = None;
                        resolution.discard_after =
                            Some(discard_events::DiscardAfter::RememberActual);
                    }
                }
                Instruction::CreateEmblem(factory) => self.create_emblem(item.controller, factory),
                Instruction::GrantNextNoncreatureArtifactAffinity => {
                    self.grant_next_noncreature_artifact_affinity(item.controller);
                }
                Instruction::GrantNextSpellUncounterable => {
                    self.grant_next_spell_uncounterable(item.controller)
                }
                Instruction::GrantGraveyardSpellsAndExileCardsThisTurn => {
                    self.grant_graveyard_spells_and_exile_cards(item.controller)
                }
                Instruction::EndTurn => {
                    self.end_turn_effect();
                    return;
                }
                Instruction::WinGame => {
                    self.try_win_game(item.controller);
                    if self.state.outcome.is_some() {
                        self.state.resolving = None;
                        return;
                    }
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
                    let lost = self.allowed_game_losses(lost);
                    self.chosen_player_loss_triggers(lost);
                    self.commit_game_losses(lost);
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
                Instruction::ExchangeLifeTotalsWithTarget {
                    draw_controller_loss,
                } => {
                    if let Some(target) = self.program_players(PlayerSet::Targeted, &item).next() {
                        self.exchange_life_totals(item.controller, target, draw_controller_loss);
                    }
                }
                Instruction::SetControllerLifeToHalfStarting => {
                    let half = STARTING_LIFE / 2 + STARTING_LIFE % 2;
                    let life = self.state.players[item.controller.index()].life;
                    match life.cmp(&half) {
                        std::cmp::Ordering::Less => {
                            let amount = half
                                .checked_sub(life)
                                .capacity("life gain amount exhausted");
                            self.gain_life(item.controller, amount);
                        }
                        std::cmp::Ordering::Greater => self.lose_life(item.controller, life - half),
                        std::cmp::Ordering::Equal => {}
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
                    let gains: Vec<_> = self
                        .program_players(set, &item)
                        .map(|player| (player, self.plan_life_gain(player, amount)))
                        .collect();
                    for (player, gain) in gains {
                        self.commit_life_gain(player, gain);
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
                Instruction::TapPermanentGroup(filters) => {
                    let recipients = self.battlefield_group(&item, filters);
                    self.tap_objects_as(item.controller, &recipients);
                }
                Instruction::UntapPermanentGroup(filters) => {
                    let recipients = self.battlefield_group(&item, filters);
                    self.untap_objects(&recipients, untap_rules::UntapCause::Effect);
                }
                Instruction::UntapSubtype {
                    players,
                    subtype,
                    other,
                } => {
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    let recipients: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && players.contains(&object.controller)
                                && (!other
                                    || item.source.is_none_or(|source| {
                                        source.object != self.object_ref(*id as CardId)
                                    }))
                                && self.has_subtype(*id as CardId, subtype)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
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
                Instruction::RemovePlayerCreatureKeywords { players, keywords } => {
                    if let Some(source) = item.source {
                        let recipients = self.player_creature_recipients(players, &item);
                        self.remove_group_keywords(source.object, recipients, keywords, None);
                    }
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
                        self.tap_objects_as(item.controller, &ids);
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
                            self.tap_objects_as(
                                item.controller,
                                &[item
                                    .source
                                    .expect("instruction requires an object source")
                                    .id],
                            );
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
                            let amount = self.nonbasic_land_count(player);
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
                Instruction::DamageByTargetPower { power_group } => {
                    let amount = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .target_card_facts
                        .as_ref()
                        .and_then(|facts| facts.iter().find(|fact| fact.group == power_group))
                        .map_or(0, |fact| fact.power.max(0));
                    let source = self.program_damage_source(&item);
                    let events: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .map(|bound| DamageEvent {
                            preventable: true,
                            source,
                            target: bound.target,
                            amount,
                            combat: false,
                        })
                        .collect();
                    self.apply_damage_batch(&events);
                }
                Instruction::DiscardDamagedPlayers(_)
                | Instruction::DamagedPlayersCannotGainLifeForGame => {
                    let resolution = self.state.resolving.as_mut().unwrap();
                    let players = resolution
                        .remembered_damage_players
                        .take()
                        .expect("remembered damage must precede its continuation");
                    for player in [Player::P0, Player::P1] {
                        if players & (1 << player.index()) == 0 {
                            continue;
                        }
                        match instruction {
                            Instruction::DiscardDamagedPlayers(amount) => {
                                resolution.discards[player.index()] = usize::from(amount);
                            }
                            Instruction::DamagedPlayersCannotGainLifeForGame => {
                                self.state.players[player.index()].cannot_gain_life_for_game = true;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Instruction::ReturnChosenCreatureTypeFromGraveyard => {
                    self.return_resolution_chosen_type(item.controller)
                }
                Instruction::EnteringPowerDamagePlayers(players) => {
                    self.event_object_power_damage_players(&item, players)
                }
                Instruction::DamagePlayersAndTheirCreatures { players, .. }
                | Instruction::DamagePlayerCreatures { players, .. }
                | Instruction::DamagePlayerPermanents { players, .. } => {
                    let (include_players, kinds, amount) = match instruction {
                        Instruction::DamagePlayerCreatures { amount, .. } => {
                            (false, 1, AmountValue::Fixed(amount))
                        }
                        Instruction::DamagePlayersAndTheirCreatures { amount, .. } => {
                            (true, 1, amount)
                        }
                        Instruction::DamagePlayerPermanents { kinds, amount, .. } => {
                            (false, kinds, amount)
                        }
                        _ => unreachable!(),
                    };
                    let amount = i128::try_from(self.item_amount(amount, &item))
                        .capacity("damage amount representation exhausted");
                    let source = self.program_damage_source(&item);
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    let targets = players
                        .iter()
                        .copied()
                        .filter(|_| include_players)
                        .map(Target::Player)
                        .chain(
                            self.state
                                .objects
                                .iter()
                                .enumerate()
                                .filter(|(id, o)| {
                                    o.zone == Zone::Battlefield
                                        && players.contains(&o.controller)
                                        && self.current_kinds(*id as CardId) & kinds != 0
                                })
                                .map(|(id, _)| Target::Card(id as CardId)),
                        );
                    let events: Vec<_> = targets
                        .map(|target| DamageEvent {
                            preventable: true,
                            source,
                            target,
                            amount,
                            combat: false,
                        })
                        .collect();
                    self.apply_damage_batch(&events);
                }
                Instruction::GrantKeywordsToUnchosenTargets { group, keywords } => {
                    let chosen = self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .remembered_objects
                        .as_ref()
                        .and_then(|objects| objects.first())
                        .copied();
                    if let (Some(source), Some(chosen)) = (item.source, chosen) {
                        let recipients = item
                            .targets
                            .iter()
                            .filter_map(|bound| {
                                bound
                                    .object
                                    .filter(|object| {
                                        *object != chosen
                                            && bound.group == group
                                            && self.on_battlefield(object.card)
                                            && self.object_ref(object.card) == *object
                                    })
                                    .map(|object| object.card)
                            })
                            .collect::<Vec<_>>();
                        self.grant_keyword_effects(&recipients, source.object, keywords);
                    }
                }
                Instruction::Damage(_)
                | Instruction::DamageRememberPlayers(_)
                | Instruction::DamageRememberPlayersScaled(_)
                | Instruction::DamageScaled(_)
                | Instruction::DamageByEnteringPower
                | Instruction::DamagePlayers(_, _)
                | Instruction::DamagePlayersScaled(_, _)
                | Instruction::DamageDiscardAmount(_) => {
                    let amount = match instruction {
                        Instruction::Damage(amount)
                        | Instruction::DamageRememberPlayers(amount)
                        | Instruction::DamagePlayers(_, amount) => i128::from(amount),
                        Instruction::DamageScaled(value)
                        | Instruction::DamageRememberPlayersScaled(value)
                        | Instruction::DamagePlayersScaled(_, value) => {
                            i128::try_from(self.item_amount(value, &item))
                                .capacity("damage amount representation exhausted")
                        }
                        Instruction::DamageByEnteringPower => self
                            .resolved_stat_value(&item, ResolvedStatValue::EnteringPower)
                            .unwrap_or(0)
                            .max(0),
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
                    if matches!(
                        instruction,
                        Instruction::DamageRememberPlayers(_)
                            | Instruction::DamageRememberPlayersScaled(_)
                    ) {
                        self.state
                            .resolving
                            .as_mut()
                            .unwrap()
                            .remembered_damage_players = Some(0);
                        self.apply_damage_batch_recording(&events, Some(item.id));
                    } else {
                        self.apply_damage_batch(&events);
                    }
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
                Instruction::GrowEventPowerToughnessDifference => {
                    let event = item
                        .entering_object
                        .or_else(|| item.event_object.as_ref().map(|event| event.object));
                    if let Some(object) = event.filter(|object| {
                        self.on_battlefield(object.card)
                            && self.object_ref(object.card) == *object
                            && self.is_creature(object.card)
                    }) {
                        let id = object.card;
                        let stats = self.stats(id);
                        let amount = i128::try_from(stats.power.abs_diff(stats.toughness))
                            .capacity("event creature stat difference exhausted");
                        self.state.objects[id as usize].growth += Stats {
                            power: amount,
                            toughness: amount,
                        };
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
                Instruction::GrantCreatureGraveyardCastThisTurn {
                    entry_counter,
                    additional_types,
                } => {
                    self.grant_creature_graveyard_cast_this_turn(
                        &item,
                        entry_counter,
                        additional_types,
                        false,
                    );
                }
                Instruction::GrantCreatureGraveyardForageThisTurn { entry_counter } => {
                    self.grant_creature_graveyard_cast_this_turn(&item, entry_counter, &[], true);
                }
                Instruction::AddLegendaryCreatureTypes { subject, types } => {
                    self.grant_legendary_creature_types(&item, subject, types)
                }
                Instruction::GrantAllCreatureTypes(subject) => {
                    self.grant_all_creature_types(&item, subject)
                }
                Instruction::GrantAllBasicLandTypesUntilCleanup => {
                    self.grant_all_basic_land_types_until_cleanup(&item);
                }
                Instruction::AddBasicLandType { subject, kind } => {
                    self.add_basic_land_type(&item, subject, kind, None);
                }
                Instruction::AddBasicLandTypeWhileCounter {
                    subject,
                    kind,
                    counter,
                } => {
                    self.add_basic_land_type(&item, subject, kind, Some(counter));
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
                Instruction::GrantTargetTriggerUntilEndOfTurn(ability) => {
                    let recipients = self.keyword_subjects(&item, Subject::Targets);
                    if !recipients.is_empty() {
                        self.grant_temporary_ability_to(&item, ability, &recipients);
                    }
                }
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
                Instruction::GrantTargetPermanentTrigger(ability) => {
                    let recipients = self.keyword_subjects(&item, Subject::Targets);
                    if !recipients.is_empty() {
                        let timestamp = self.next_role_timestamp();
                        for recipient in recipients {
                            let object = self.object_ref(recipient);
                            self.grant_permanent_triggers(&item, object, &[ability], timestamp);
                        }
                    }
                }
                Instruction::GrantPermanentSelfAbility(kind) => {
                    self.grant_permanent_self_ability(&item, kind)
                }
                Instruction::GrantCastCreatureKeywords(keywords) => {
                    self.grant_cast_creature_keywords(&item, keywords);
                }
                Instruction::GrantCastSpellHaste | Instruction::GrantCastSpellKeywords(_) => {
                    let keywords = match instruction {
                        Instruction::GrantCastSpellKeywords(keywords) => keywords,
                        _ => Keywords::HASTE,
                    };
                    if let Some(spell) = self.live_cast_spell(&item) {
                        self.grant_stack_keywords(spell, keywords);
                    }
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
                Instruction::GrantKeywordsUntilNextTurn { subject, keywords } => {
                    let ids = self.keyword_subjects(&item, subject);
                    self.grant_keywords_until_next_turn(
                        &ids,
                        item.source
                            .expect("instruction requires an object source")
                            .object,
                        item.controller,
                        keywords,
                    );
                }
                Instruction::RemoveKeywords { subject, keywords }
                | Instruction::RemoveKeywordsUntilNextTurn { subject, keywords } => {
                    let recipients = self.keyword_subjects(&item, subject);
                    let next_turn =
                        matches!(instruction, Instruction::RemoveKeywordsUntilNextTurn { .. })
                            .then_some(item.controller);
                    self.remove_group_keywords(
                        item.source.expect("keyword removal has a source").object,
                        recipients,
                        keywords,
                        next_turn,
                    );
                }
                Instruction::GrantControlledTokenKeywordsUntilNextTurn(keywords) => {
                    let ids = self
                        .controlled_token_ids(item.controller)
                        .collect::<Vec<_>>();
                    self.grant_keywords_until_next_turn(
                        &ids,
                        item.source.expect("keyword grant has a source").object,
                        item.controller,
                        keywords,
                    );
                }
                Instruction::GrantPermanentGroupKeywords { filters, keywords }
                | Instruction::GrantOtherPermanentGroupKeywords { filters, keywords } => {
                    self.grant_permanent_group_keywords(
                        &item,
                        filters,
                        keywords,
                        matches!(
                            instruction,
                            Instruction::GrantOtherPermanentGroupKeywords { .. }
                        ),
                    );
                }
                Instruction::GrantCreatures { filter, keywords }
                | Instruction::GrantCreatureKeywordAbilities { filter, keywords }
                | Instruction::GrantCreaturesWithPowerAtLeast {
                    filter, keywords, ..
                }
                | Instruction::GrantCreaturesWithPowerAtMost {
                    filter, keywords, ..
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
                            ) && match instruction {
                                Instruction::GrantCreaturesWithPowerAtLeast { minimum, .. } => {
                                    self.stats(id).power >= i128::from(minimum)
                                }
                                Instruction::GrantCreaturesWithPowerAtMost { maximum, .. } => {
                                    self.stats(id).power <= i128::from(maximum)
                                }
                                _ => true,
                            }
                        })
                        .collect();
                    let source = item
                        .source
                        .expect("instruction requires an object source")
                        .object;
                    if matches!(
                        instruction,
                        Instruction::GrantCreatureKeywordAbilities { .. }
                    ) {
                        self.grant_keyword_abilities(&affected, source, keywords);
                    } else {
                        self.grant_keyword_effects(&affected, source, keywords);
                    }
                }
                Instruction::GrantRememberedCreatures(keywords)
                | Instruction::GrantRememberedCreaturesUntilNextTurn(keywords) => {
                    let objects = self
                        .state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .remembered_objects
                        .take();
                    let affected: Vec<_> = objects
                        .into_iter()
                        .flatten()
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
                    if matches!(instruction, Instruction::GrantRememberedCreatures(_)) {
                        self.grant_keyword_effects(&affected, source, keywords);
                    } else {
                        self.grant_keywords_until_next_turn(
                            &affected,
                            source,
                            item.controller,
                            keywords,
                        );
                    }
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
                Instruction::GrowFilteredScaled {
                    filter,
                    amount,
                    power,
                    toughness,
                } => {
                    self.grow_filtered_scaled(&item, filter, amount, power, toughness);
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
                Instruction::GrantChosenCreatureTypeKeywords { other, keywords } => {
                    self.grant_chosen_creature_type_keywords(&item, other, keywords)
                }
                Instruction::ModifyPlayerCreatures {
                    players,
                    power,
                    toughness,
                    keywords,
                } => {
                    let recipients = self.player_creature_recipients(players, &item);
                    for &id in &recipients {
                        self.state.objects[id as usize].growth += Stats {
                            power: i128::from(power),
                            toughness: i128::from(toughness),
                        };
                    }
                    if let Some(source) = item.source {
                        self.grant_keyword_effects(&recipients, source.object, keywords);
                    }
                }
                Instruction::RemoveAbilitiesAndSetCreatureBaseUntilNextTurn {
                    power,
                    toughness,
                } => {
                    self.suppress_targets_and_set_creature_base_until_next_turn(
                        &item, power, toughness,
                    );
                }
                Instruction::RemoveAbilities { duration, .. }
                | Instruction::RemovePlayerCreatureAbilities { duration, .. } => {
                    if let Some(source) = item.source {
                        let recipients = match instruction {
                            Instruction::RemoveAbilities { subject, .. } => {
                                self.keyword_subjects(&item, subject)
                            }
                            Instruction::RemovePlayerCreatureAbilities { players, .. } => {
                                self.player_creature_recipients(players, &item)
                            }
                            _ => unreachable!("ability removal recipients"),
                        };
                        let recipients: Vec<_> = recipients
                            .into_iter()
                            .filter(|&id| self.on_battlefield(id))
                            .map(|id| self.object_ref(id))
                            .collect();
                        self.remove_abilities_exact(
                            &recipients,
                            source.object,
                            item.controller,
                            duration,
                        );
                    }
                }
                Instruction::SetPlayerCreatureCharacteristics {
                    players,
                    additional_types,
                    power,
                    toughness,
                    remove_abilities,
                } => self.set_player_creature_characteristics(
                    &item,
                    players,
                    additional_types,
                    power,
                    toughness,
                    remove_abilities,
                ),
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
                        self.state.resolving.as_mut().unwrap().remembered_objects = (!affected
                            .is_empty())
                        .then(|| affected.iter().map(|(object, _)| *object).collect());
                    }
                    self.effect_counters(affected, replacements::ResumeMode::Program);
                }
                Instruction::PutPowerExcessCounters { filter } => {
                    self.put_power_excess_counters(&item, filter);
                }
                Instruction::PutPermanentGroupCounters {
                    filters,
                    kind,
                    amount,
                } => {
                    let changes = self
                        .battlefield_group(&item, filters)
                        .into_iter()
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::PutCreatureGroupCounters {
                    filters,
                    kind,
                    amount,
                } => {
                    let source = item.source.expect("creature group source").object;
                    let query = self.characteristic_query();
                    let changes = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            filters.iter().any(|&filter| {
                                query.creature_matches(filter, item.controller, source, id)
                            })
                        })
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
                }
                Instruction::ChangeControlledPermanentGroupCounters { filters, effect }
                | Instruction::ChangeNewlyEnteredPermanentGroupCounters { filters, effect } => {
                    self.change_controlled_permanent_group_counters(
                        item.controller,
                        filters,
                        effect,
                        matches!(
                            instruction,
                            Instruction::ChangeNewlyEnteredPermanentGroupCounters { .. }
                        ),
                    );
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
                    let ids = self.linked_exile_for_item(&item);
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
                    self.exile_grave_targets_linked(&item, None);
                }
                Instruction::ExileGraveTargetsLinkedGrant { creature_reflexive } => {
                    self.exile_grave_targets_linked(&item, Some(creature_reflexive));
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
                Instruction::RemoveCounter { subject, kind } => {
                    for id in self.keyword_subjects(&item, subject) {
                        if self.on_battlefield(id) && self.counter_count(id, kind) > 0 {
                            self.pay_counters(id, &Counters::of(kind, 1));
                        }
                    }
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
                Instruction::PutCountersOnEnteredCreatures {
                    players,
                    kind,
                    amount,
                } => {
                    let changes = self
                        .player_creature_recipients(players, &item)
                        .into_iter()
                        .map(|id| self.object_ref(id))
                        .filter(|&object| self.entered_this_turn(object))
                        .map(|object| (object, Counters::of(kind, u64::from(amount))))
                        .collect();
                    self.effect_counters(changes, replacements::ResumeMode::Program);
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
                Instruction::PutCountersOnTargetsByController {
                    relation,
                    kind,
                    amount,
                } => {
                    let changes = self
                        .keyword_subjects(&item, Subject::Targets)
                        .into_iter()
                        .filter(|&id| {
                            self.on_battlefield(id)
                                && match relation {
                                    CastRelation::You => {
                                        self.state.objects[id as usize].controller
                                            == item.controller
                                    }
                                    CastRelation::Opponent => {
                                        self.state.objects[id as usize].controller
                                            != item.controller
                                    }
                                    CastRelation::Any => true,
                                }
                        })
                        .map(|id| (self.object_ref(id), Counters::of(kind, u64::from(amount))))
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
                | Instruction::PutCountersThenScheduleEndStep { .. }
                | Instruction::PutCountersScaled { .. }
                | Instruction::PutCountersMixed { .. } => {
                    let (subject, counters) = match instruction {
                        Instruction::PutCounters {
                            subject,
                            kind,
                            amount,
                        }
                        | Instruction::PutCountersThenScheduleEndStep {
                            subject,
                            kind,
                            amount,
                            ..
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
                    if let Instruction::PutCountersThenScheduleEndStep { kind, ability, .. } =
                        instruction
                    {
                        self.effect_counters_with_delayed_step(&item, changes, kind, ability);
                    } else {
                        self.effect_counters(changes, replacements::ResumeMode::Program);
                    }
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
                Instruction::GainControlWhileEventAuraAttached => {
                    self.gain_control_while_event_aura_attached(&item);
                }
                Instruction::RemoveTargetAbilitiesUntilEndOfTurn => {
                    self.remove_target_abilities_until_end_of_turn(&item);
                }
                Instruction::BlankTargetLandWhileBlighted => {
                    self.resolve_source_linked_effect(
                        &item,
                        SourceLinkedEffectKind::BlankLandWhileBlighted,
                    );
                }
                Instruction::RemoveTargetAbilitiesUntilSourceLeaves => {
                    self.resolve_source_linked_effect(
                        &item,
                        SourceLinkedEffectKind::RemoveAbilities,
                    );
                }
                Instruction::MakeTargetsTreasureWhileSourceRemains => {
                    self.resolve_source_linked_effect(&item, SourceLinkedEffectKind::Treasure);
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
                Instruction::TapOrUntap => {
                    if let [bound] = item.targets.as_slice()
                        && self.same_target_object(*bound)
                        && let Target::Card(id) = bound.target
                    {
                        if self.state.objects[id as usize].tapped {
                            self.untap_objects(&[id], untap_rules::UntapCause::Effect);
                        } else {
                            self.tap_objects_as(item.controller, &[id]);
                        }
                    }
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
                            self.tap_objects_as(item.controller, &[id]);
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
                        self.tap_objects_as(item.controller, &ids);
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
                Instruction::Suspect { subject } => {
                    self.set_suspected_subject(&item, subject, true)
                }
                Instruction::Unsuspect { subject } => {
                    self.set_suspected_subject(&item, subject, false)
                }
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
                Instruction::ReturnGraveyardTargetsAnimated {
                    animation,
                    x_counters,
                } => {
                    self.return_graveyard_targets_animated(&item, animation, x_counters);
                }
                Instruction::ReturnToBattlefieldWithCounters(amounts) => {
                    self.return_targets_with_counter_events(
                        &item,
                        amounts,
                        Zone::Graveyard,
                        false,
                        false,
                    );
                }
                Instruction::ReturnExiledTargetTappedWithCounters(amounts) => {
                    self.return_targets_with_counter_events(
                        &item,
                        amounts,
                        Zone::Exile,
                        true,
                        false,
                    );
                }
                Instruction::ReturnToBattlefieldThenCounters(amounts) => {
                    self.return_targets_with_counter_events(
                        &item,
                        amounts,
                        Zone::Graveyard,
                        false,
                        true,
                    );
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
                Instruction::ReturnTargetAttachmentToControlledCreature => {
                    if self.return_target_attachment(&item) {
                        return;
                    }
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
                Instruction::ReturnToBattlefieldAttacking(players) => {
                    let defender = self
                        .program_players(players, &item)
                        .next()
                        .map(CombatDefender::Player);
                    let moves = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .filter_map(|bound| match bound.target {
                            Target::Card(id) => {
                                let mut movement = self.entry_move(id, Zone::Battlefield, true);
                                movement.controller = item.controller;
                                movement.attacking = defender;
                                Some(movement)
                            }
                            _ => None,
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnSourceTappedAttacking => {
                    if let Some(source) = item.source {
                        self.return_graveyard_tapped_attacking(source.object, item.controller);
                    }
                }
                Instruction::ReturnTargetTappedAttacking => {
                    if let Some(id) = item.targets.iter().find_map(|&bound| {
                        if let Target::Card(id) = bound.target
                            && self.same_target_object(bound)
                            && self.state.objects[id as usize].zone == Zone::Graveyard
                        {
                            return Some(id);
                        }
                        None
                    }) {
                        self.return_graveyard_tapped_attacking(
                            self.object_ref(id),
                            item.controller,
                        );
                    }
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
                Instruction::ReturnNonlandNontokenPermanents => {
                    let ids = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter_map(|(index, object)| {
                            let id = index as CardId;
                            (object.zone == Zone::Battlefield
                                && !object.is_token()
                                && self.current_kinds(id) & 2 == 0)
                                .then_some(id)
                        })
                        .collect::<Vec<_>>();
                    let events = self.move_objects_raw(&ids, Zone::Hand);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::ReturnToHandCounted => {
                    let mut ids = Vec::new();
                    for &bound in &item.targets {
                        if self.same_target_object(bound)
                            && let Target::Card(id) = bound.target
                            && !ids.contains(&id)
                        {
                            ids.push(id);
                        }
                    }
                    let events = self.move_objects_raw(&ids, Zone::Hand);
                    let count = events
                        .iter()
                        .filter(|event| {
                            event.moved
                                && event.before.zone == Zone::Battlefield
                                && event.destination == Zone::Hand
                        })
                        .count();
                    self.state.resolving.as_mut().unwrap().moved_count =
                        Some(count.try_into().capacity("returned object count exhausted"));
                    self.finish_zone_events(&events, &[]);
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
                Instruction::ReturnGraveyardTargetsToHandOrBattlefield {
                    maximum_mana_value,
                    bargain_only,
                } => {
                    if self.start_target_graveyard_return(&item, maximum_mana_value, bargain_only) {
                        return;
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
                Instruction::ReturnCreaturesWithoutSubtype(_)
                | Instruction::ReturnCreaturesWithoutSubtypes(_) => {
                    let subtypes = match &instruction {
                        Instruction::ReturnCreaturesWithoutSubtype(subtype) => {
                            std::slice::from_ref(subtype)
                        }
                        Instruction::ReturnCreaturesWithoutSubtypes(subtypes) => *subtypes,
                        _ => unreachable!(),
                    };
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, object)| {
                            object.zone == Zone::Battlefield
                                && self.is_creature(*id as CardId)
                                && !subtypes
                                    .iter()
                                    .any(|name| self.has_subtype(*id as CardId, name))
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Hand);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::ReturnToHandDrawForTokens
                | Instruction::ReturnToHandDrawForSubtype(_) => {
                    // Capture the legal target before changing its incarnation.
                    // The subtype condition is independent of a successful move.
                    let draws = item
                        .targets
                        .iter()
                        .filter(|&&bound| {
                            self.same_target_object(bound)
                                && match bound.target {
                                    Target::Card(id) => match instruction {
                                        Instruction::ReturnToHandDrawForTokens => {
                                            self.state.objects[id as usize].is_token()
                                        }
                                        Instruction::ReturnToHandDrawForSubtype(subtype) => {
                                            self.has_subtype(id, subtype)
                                        }
                                        _ => unreachable!(),
                                    },
                                    _ => false,
                                }
                        })
                        .count();
                    self.move_target_batch(&item.targets, Zone::Hand);
                    self.draw_many(item.controller, draws as u128);
                }
                Instruction::ReturnArtifactAsRobot => {
                    let source = item.source.expect("return spell has a source");
                    let moves = item
                        .targets
                        .iter()
                        .filter(|&&bound| self.same_target_object(bound))
                        .filter_map(|bound| bound.object)
                        .map(|object| {
                            let mut movement =
                                self.entry_move(object.card, Zone::Battlefield, false);
                            movement.controller = item.controller;
                            if !movement.before.is_creature() {
                                movement.return_modification =
                                    Some((ReturnKind::FlyingRobot, source.object));
                            }
                            movement
                        })
                        .collect();
                    self.stage_entries(moves, None);
                }
                Instruction::ReturnToBattlefieldAsSkeletonWithoutAbilities => {
                    self.return_targets_as_skeletons(&item);
                }
                Instruction::DestroyTargetsUnlessGreaterCreaturePower => {
                    self.destroy_targets_unless_greater_creature_power(&item);
                }
                Instruction::DestroyTargetsAtMostPower(maximum) => {
                    let targets: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| {
                            let Target::Card(id) = bound.target else {
                                return false;
                            };
                            self.same_target_object(bound)
                                && self.on_battlefield(id)
                                && self.is_creature(id)
                                && self.stats(id).power <= i128::from(maximum)
                                && !self.effect_destruction_prevented(id)
                        })
                        .collect();
                    self.move_target_batch(&targets, Zone::Graveyard);
                }
                Instruction::DestroyCounted { kinds } => {
                    let ids: Vec<_> = item
                        .targets
                        .iter()
                        .filter_map(|&bound| match bound.target {
                            Target::Card(id)
                                if self.same_target_object(bound)
                                    && self.on_battlefield(id)
                                    && !self.effect_destruction_prevented(id) =>
                            {
                                Some(id)
                            }
                            _ => None,
                        })
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Graveyard);
                    let count = events
                        .iter()
                        .filter(|e| e.moved && e.before.kinds & kinds != 0)
                        .count();
                    self.state.resolving.as_mut().unwrap().moved_count = Some(count as u64);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::DestroyThenRememberControlledOrToken => {
                    let events = self.destroy_targets_outcome(&item.targets);
                    let qualifies = events.iter().any(|event| {
                        event.moved
                            && (event.before.controller == item.controller
                                || event.before.is_token())
                    });
                    self.state.resolving.as_mut().unwrap().moved_count = Some(u64::from(qualifies));
                }
                Instruction::DestroyAllTargets => {
                    self.destroy_targets_outcome(&item.targets);
                }
                Instruction::ExileAllTargets => {
                    let mut seen = std::collections::BTreeSet::new();
                    let targets: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|target| target.object.is_some_and(|object| seen.insert(object)))
                        .collect();
                    self.move_target_batch(&targets, Zone::Exile);
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn resolution_chosen_color(&self) -> Option<crate::cards::ChoiceColor> {
        self.state.resolving.as_ref().and_then(|r| r.chosen_color)
    }
}
