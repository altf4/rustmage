//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;
use crate::cards::{CastFilter, CastTarget, StaticAbilityFragment};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TargetCardFacts {
    pub group: u16,
    pub controller: Player,
    pub mana_value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub cascade: Option<Box<cascade::CascadeView>>,
    #[allow(clippy::box_collection)]
    pub target_card_facts: Option<Box<Vec<TargetCardFacts>>>,
    #[allow(clippy::box_collection)]
    pub immediate_blink_return: Option<Box<Vec<ReturnLinkView>>>,
    pub connive: Option<Box<super::connive::ConniveSequence>>,
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
    pub discard_after: Option<discard_events::DiscardAfter>,
    pub discards: [usize; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub milled: Vec<ObjectRef>,
    pub milled_selection: Option<Box<super::milled_selection::MilledSelection>>,
    pub library: Option<LibraryChoice>,
    pub selection: Option<Box<super::library_selection::LibrarySelection>>,
    pub return_attachment: Option<Box<super::return_attachment::ReturnAttachment>>,
    pub search: Option<Box<super::search::SearchChoice>>,
    pub optional_searches: Option<Box<super::search::OptionalSearchSequence>>,
    pub choice: Option<Box<super::effect_choices::EffectChoice>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LibraryChoice {
    pub surveil: bool,
    pub remaining: Vec<CardId>,
    pub top: Vec<CardId>,
    pub bottom: Vec<CardId>,
    pub graveyard: Vec<CardId>,
}

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
        let id = match subject {
            Subject::Entering => self.entering_subject(item),
            Subject::Source => self
                .on_battlefield(item.card)
                .then_some(item.card)
                .filter(|&id| self.object_ref(id) == item.object),
            Subject::Attached => self.attached_subject(item),
            Subject::Targets => match item.targets.as_slice() {
                [bound] if self.same_target_object(*bound) => match bound.target {
                    Target::Card(id) => Some(id),
                    _ => None,
                },
                _ => None,
            },
        };
        id.is_some_and(|id| self.creature_matches(filter, item.controller, item.object, id))
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
            LibraryDestination::Shuffle | LibraryDestination::SecondFromTop => {
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
        // Remove the looked-at suffix once, then preserve the untouched middle.
        for &card in &choice.graveyard {
            self.move_object(card, Zone::Graveyard);
        }
        let library = &mut self.state.players[player.index()].library;
        let count = choice.top.len() + choice.bottom.len();
        let retained = library.len() - count;
        library.truncate(retained);
        // Reuse library capacity; only bottom insertion shifts the untouched middle.
        library.splice(0..0, choice.bottom.iter().rev().copied());
        library.extend(choice.top.iter().rev().copied());
        self.run_program();
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
                        )
                    })
            })
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::CounterBatchPlacedByYou { kind, filter } =
                    ability.condition.event()
                else {
                    continue;
                };
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
        self.temporary_departure_triggers(event);
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
                        TriggerCondition::Sacrificed {
                            filter: f,
                            own_turn,
                        } => {
                            event.sacrificed_by == Some(source.controller)
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
                        if ability
                            .program
                            .iter()
                            .any(|i| matches!(i, Instruction::ReturnEventCardToHand))
                        {
                            self.state.pending_triggers.last_mut().unwrap().return_link =
                                Some(Box::new(ReturnLinkView {
                                    object: event.after,
                                    zone: Zone::Graveyard,
                                    controller: event.before.owner,
                                    counter: None,
                                    tapped: false,
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
        if sources.is_empty() {
            return;
        }
        let entered = self.object_view(event.after.card);
        // This prohibition concerns the event, irrespective of which permanent
        // would observe it. Death/attack/step events take their separate paths.
        if entered.kinds & entry_suppression != 0 {
            return;
        }
        for source in sources {
            let same = source.reference == event.after;
            let yours = source.controller == entered.controller;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::SelfDiesWithCounters
                    | TriggerCondition::InZone { .. }
                    | TriggerCondition::ManaSpent { .. }
                    | TriggerCondition::Granted(_)
                    | TriggerCondition::OncePerTurn(_)
                    | TriggerCondition::DuringOpponentsTurn(_)
                    | TriggerCondition::When { .. }
                    | TriggerCondition::InterveningIf { .. } => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::SelfEntersIfCast => same && event.was_cast,
                    TriggerCondition::SelfOffspring => same && event.offspring_paid,
                    TriggerCondition::SelfKicked => same && event.kicked,
                    TriggerCondition::SelfGiftEnters => same && event.gift.is_some(),
                    TriggerCondition::SelfBargained => same && event.bargained,
                    TriggerCondition::SelfEvidence => same && event.evidence_collected,
                    TriggerCondition::EvidenceCollected | TriggerCondition::Chapter { .. } => false,
                    TriggerCondition::SelfEnters
                    | TriggerCondition::SelfEntersOrAttacks
                    | TriggerCondition::SelfEntersOrAttacksWhileSaddled
                    | TriggerCondition::SelfEntersOrDies => same,
                    TriggerCondition::OtherControlledNonlandReturnedToOwnersHand
                    | TriggerCondition::ControlledAbilityTargetsPlayerOrPermanent
                    | TriggerCondition::TokensEnterYou
                    | TriggerCondition::CardsLeaveGraveyard
                    | TriggerCondition::CreatureExploresYou
                    | TriggerCondition::CreatureExploresCardYou { .. }
                    | TriggerCondition::Foraged
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Reflexive
                    | TriggerCondition::Ward
                    | TriggerCondition::SelfTargeted { .. }
                    | TriggerCondition::CardDrawn { .. }
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
                    | TriggerCondition::SelfLeaves
                    | TriggerCondition::ReturnDeath(_)
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::AttachedCreatureDies
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesWithMaximumStat { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::CreatureDiesAttackState { .. }
                    | TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::LifeLost { .. }
                    | TriggerCondition::SelfCountersAdded { .. }
                    | TriggerCondition::CreatureAttacksYou { .. }
                    | TriggerCondition::OtherCreatureCountersAdded { .. }
                    | TriggerCondition::OtherCreatureCounterPlacedByYou { .. }
                    | TriggerCondition::CounterPlacedByYou { .. }
                    | TriggerCondition::CounterBatchPlacedByYou { .. }
                    | TriggerCondition::AttackDeclared { .. }
                    | TriggerCondition::AttackPlayer(_)
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SelfAttacksIfEquipped
                    | TriggerCondition::SelfFirstAttack
                    | TriggerCondition::SelfAttacksAlone
                    | TriggerCondition::ControlledCreatureAttacksAlone
                    | TriggerCondition::ControlledEquippedCreatureAttacks
                    | TriggerCondition::SelfAttacksWhileSaddled
                    | TriggerCondition::SelfSaddledFirst
                    | TriggerCondition::SelfPaysTeamwork
                    | TriggerCondition::EquippedAttacks
                    | TriggerCondition::EquippedAttacksAlone
                    | TriggerCondition::GrantedAttacks
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
                                EntryPredicate::ArtifactManaValueAtLeast(minimum) => {
                                    entered.kinds & 16 != 0
                                        && entered.card.fixed_mana_value() >= u64::from(minimum)
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
                    TriggerCondition::ArtifactEntersYou => yours && entered.kinds & 16 != 0,
                    TriggerCondition::EnchantmentEntersYou => yours && entered.kinds & 32 != 0,
                    TriggerCondition::CreatureEntersYou => yours && entered.is_creature(),
                    TriggerCondition::LandEntersYou => yours && entered.kinds & 2 != 0,
                    TriggerCondition::OtherArtifactEntersYou => {
                        yours && !same && entered.kinds & 16 != 0
                    }
                };
                if matched && self.queue_zone_trigger(source, index, event) {
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
            gift: None,
            source_move: None,
            borrowed_slot: None,
            target_controller_lki: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
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
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            attachment_trigger: event,
            cast: None,
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_transform: None,
            source_card: source.card,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            source_facts: DamageFacts::from_view(source),
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
            source_counter_lki: card_quantities::counter_quantity_facts(
                definition.triggers()[index].program,
                |kind| source.counters.get(kind),
            ),
            event_object: None,
            source_quantity_lki: self
                .source_quantity_facts(definition.triggers()[index].program, source),
            sacrificed_toughness: None,
            source_attachment: source.attached_to,
            card: source.id,
            object: source.reference,
            controller: source.controller,
            targets: Vec::new(),
            mode: None,
        });
        true
    }

    /// A spell becomes cast only after every declaration and cost is complete.
    /// Completed insertion paths share this hook; abilities never count as casts.
    pub(super) fn push_stack_item(&mut self, mut item: SpellView) {
        self.capture_transform_stack_serial(&mut item);
        let cast = (item.kind == StackKind::Spell && item.cast.is_some() && !item.is_copy)
            .then_some((item.controller, item.source_card));
        if cast.is_some() {
            self.complete_prepared_cast(item.object);
        }
        self.register_stack_graveyard_exile(&item);
        self.state.stack.push(item);
        let Some((caster, card)) = cast else { return };
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
            CastFilter::NotOwnedByYou => {
                spell_owner.is_some_and(|owner| owner != source.controller)
            }
            CastFilter::SourceChosenColor => {
                matches!(source.entry_choice, Some(EntryChoiceValue::Color(color)) if card.colors() & (1 << color.index()) != 0)
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
        self.consume_turn_cast_grants(caster, card);
        let first_trigger = self.state.pending_triggers.len();
        let spell_owner =
            stack.map(|i| self.state.objects[self.state.stack[i].card as usize].owner);
        let count = &mut self.state.players[caster.index()].spells_cast_this_turn;
        *count = count.checked_add(1).capacity("spell count exhausted");
        let ordinal = *count;
        if CastFilter::InstantSorcery.matches(card) {
            let count = &mut self.state.players[caster.index()].instant_sorcery_cast_this_turn;
            *count = count
                .checked_add(1)
                .capacity("qualifying spell count exhausted");
        }
        let cast_event = stack.and_then(|i| {
            let item = &self.state.stack[i];
            item.cast.map(|cast| CastPaymentEvent {
                mana_value: item.mana_value(),
                stack_id: item.id,
                object: item.object,
                card: item.source_card,
                controller: item.controller,
                payment: cast.payment,
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
                        && self.cast_payment_condition(
                            ability.condition,
                            source.reference,
                            cast_event.as_ref(),
                        )
                        && (!ability.condition.requires_opponents_turn()
                            || self.state.active != source.controller)
                        && self.cast_filter_matches_value(
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
                        && wanted.is_none_or(|wanted| wanted == ordinal)
                        && self.queue_trigger_from(&source, index, grant.map(Box::new))
                    {
                        // A cast trigger owns the triggering spell's chosen X even after that
                        // spell is countered or leaves the stack. ChosenX reads this packet.
                        let chosen_x = stack.and_then(|i| self.state.stack[i].x);
                        self.state.pending_triggers.last_mut().unwrap().x = chosen_x;
                        if ability.condition.mana_spent_rule().is_some()
                            || cast_symbol_quantities::selected(ability.program)
                            || ability.program.iter().any(|instruction| {
                                matches!(
                                    instruction,
                                    Instruction::GainLifeScaled(_, AmountValue::SpellColors)
                                        | Instruction::DamageScaled(
                                            AmountValue::CastSpellManaValue
                                        )
                                        | Instruction::CreateTokensWithTemporaryKeywords { .. }
                                        | Instruction::DamagePlayers(PlayerSet::CastingPlayer, _)
                                        | Instruction::PutCountersScaled {
                                            amount: AmountValue::SpellColors,
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
                let source = self.object_view(item.card);
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
    ) {
        self.attachment_attack_triggers(actor, alone);
        self.temporary_attack_triggers(actor);
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

    /// Reuse the live attachment index once per completed declaration. No arena scan
    /// or mutable granted-ability graph; every queued packet owns its provenance.
    fn attachment_attack_triggers(&mut self, actor: Player, alone: Option<ObjectRef>) {
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
                    || !self
                        .state
                        .attackers
                        .iter()
                        .any(|a| a.card == recipient.card)
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
                    TriggerCondition::EquippedAttacks => false,
                    TriggerCondition::EquippedAttacksAlone if alone == Some(recipient) => false,
                    TriggerCondition::GrantedAttacks => true,
                    TriggerCondition::SelfAttacks if ability.condition.is_granted() => true,
                    _ => continue,
                };
                // Supported games have two players, owner==controller, and only
                // opposing player/planeswalker defenders. Both share this player.
                let event = Box::new(AttachmentTriggerView {
                    grantor: giver.reference,
                    granting_card: giver.card,
                    recipient,
                    defender: Some(actor.other()),
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
        if let StackKind::TriggeredAbility { ability } = item.kind
            && (!self.cast_payment_condition(
                item.trigger_definition().triggers()[ability as usize].condition,
                item.object,
                item.cast_event.as_deref(),
            ) || !self.trigger_guards_hold(
                item.trigger_definition().triggers()[ability as usize].condition,
                item.controller,
                item.object,
                item.source_tapped,
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
        let valid = !was_targeted || !item.targets.is_empty();
        if !valid {
            if item.kind == StackKind::Spell {
                if item
                    .cast
                    .is_some_and(|c| matches!(c.face, CastFace::Adventure | CastFace::Omen))
                {
                    self.move_adventure_spell(&item, Zone::Graveyard);
                } else {
                    self.move_object(item.card, Zone::Graveyard);
                }
            }
            self.give_priority();
            return;
        }
        let program = match item.kind {
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::CasualtyCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedWarpExile
            | StackKind::ActivatedAbility { .. } => {
                item.program().expect("selected ability program").1
            }
            StackKind::Spell => match item.source_card.effect() {
                Effect::Aura(_) => {
                    self.record_resolved_spell_name(&item);
                    let target = item.targets[0].object.unwrap();
                    let mut movement = self.entry_move(item.card, Zone::Battlefield, false);
                    movement.attachment = Some(target);
                    movement.cast_event = item.actual_cast_event().map(Box::new);
                    movement.cast_payment = item.cast.filter(|_| !item.is_copy).map(|c| c.payment);
                    movement.cast_origin = item.cast.filter(|_| !item.is_copy).map(|c| c.origin);
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
                    if item.source_card.other_face().is_some() {
                        self.state.objects[item.card as usize].card = item.source_card;
                    }
                    self.record_resolved_spell_name(&item);
                    if item.cast.is_some_and(|c| {
                        c.offspring == Some(true)
                            || c.kicker == Some(true)
                            || c.bargain == Some(true)
                            || c.evidence == Some(true)
                            || matches!(c.gift, Some(GiftDeclaration::Promised { .. }))
                            || c.method == CastMethod::Warp
                            || item.source_card.entry_counter_rules().iter().any(|r| {
                                matches!(
                                    r.amount,
                                    EntryCounterAmount::ColorsSpent
                                        | EntryCounterAmount::IfNotCastOrZero(_)
                                )
                            })
                    }) || item.x.is_some()
                        || self.has_cast_entry_counters(item.object)
                        || !item.source_card.hand_cast_entry_counters().is_empty()
                        || !item.source_card.cast_entry_counters().is_empty()
                        || item.source_card.entry_choice().is_some()
                        || self.has_entry_counter_replacements(item.source_card)
                    {
                        let mut movement = self.entry_move(item.card, Zone::Battlefield, false);
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
                        let mut event = self.move_object_raw(item.card, Zone::Battlefield);
                        event.cast_event = item.actual_cast_event().map(Box::new);
                        event.was_cast = item.cast.is_some() && !item.is_copy;
                        self.finish_zone_events(std::slice::from_ref(&event), &[]);
                    }
                    self.give_priority();
                    return;
                }
                Effect::Spell(..) | Effect::Modal(_) => {
                    item.source_card.spell_program(item.mode).unwrap().1
                }
                Effect::Land | Effect::Emblem | Effect::Unsupported => {
                    unreachable!("unsupported spell admitted")
                }
            },
        };
        if program
            .iter()
            .any(|i| matches!(i, Instruction::IfThisAbilityResolutionNumber { .. }))
        {
            self.record_ability_resolution(&item);
        }
        assert!(self.state.resolving.is_none());
        let target_card_facts = program
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    Instruction::LoseLifeTargetCardManaValue(_)
                        | Instruction::MayTargetController { .. }
                        | Instruction::SearchTargetController(_)
                        | Instruction::LoseLifeTargetController(_)
                        | Instruction::DamageTargetController(_)
                )
            })
            .then(|| {
                Box::new(
                    item.targets
                        .iter()
                        .filter_map(|bound| match bound.target {
                            Target::Card(card) => Some(TargetCardFacts {
                                group: bound.group,
                                controller: self.state.objects[card as usize].controller,
                                mana_value: self.state.objects[card as usize]
                                    .card
                                    .fixed_mana_value(),
                            }),
                            _ => None,
                        })
                        .collect(),
                )
            });
        self.state.resolving = Some(Resolution {
            cascade: None,
            target_card_facts,
            immediate_blink_return: None,
            explore: None,
            evidence: None,
            connive: None,
            item,
            program,
            pc: 0,
            target_group: 0,
            moved_count: None,
            discard_after: None,
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
            if self.miracle_choice_pending() {
                return;
            }
            self.finish_cascade();
            if self.advance_replacements(false) {
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
                    && self.state.objects[item.card as usize].zone == Zone::Stack
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
                    | Instruction::AttachEquipmentByGroups { .. }
                    | Instruction::ReturnAllTargetsToHand
                    | Instruction::CopyTargetSpell { .. }
                    | Instruction::CopyTargetAbility { .. }
                    | Instruction::DamageByGroup { .. }
                    | Instruction::PowerDamage { .. }
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
                Instruction::PrepareSource => self.prepare_permanent(item.object),
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
                    for id in self.keyword_subjects(&item, subject) {
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
                Instruction::ReturnLinkedExileToHand => self.return_linked_exile(item.object),
                Instruction::ScheduleSacrificedArtifactCopy { ability } => {
                    let mut delayed = item.clone();
                    delayed.id = self.next_stack_id();
                    delayed.kind = StackKind::TriggeredAbility { ability };
                    delayed.targets.clear();
                    self.state
                        .delayed_artifact_copies
                        .get_or_insert_default()
                        .push(delayed);
                }
                Instruction::CreateCapturedArtifactCopy => {
                    if let Some(recipe) = item.captured_artifact_copy.as_deref() {
                        self.stage_copy_creation(item.controller, *recipe, 1);
                    }
                }
                Instruction::CopyNextInstantSorcery { ability } => {
                    self.remember_next_spell_copy(&item, ability)
                }
                Instruction::CopyCastSpell { .. } | Instruction::CopyStormSpell => {
                    if self.copy_captured_spell(&item) {
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
                    self.grant_keyword_effects(&ids, item.object, keywords);
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
                    let count = self.amount_value(amount, item.object, item.controller);
                    if self.start_mobilize(token, count, &item) {
                        return;
                    }
                }
                Instruction::SacrificeDelayedTokens => self.sacrifice_delayed_tokens(&item),
                Instruction::ExileWarpedPermanent => self.exile_warped_permanent(&item),
                Instruction::Crew { .. } => self.resolve_crew(&item),
                Instruction::BecomeSaddled { .. } => self.resolve_saddle(&item),
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
                                item.object,
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
                        .filter(|(_, o)| o.zone == Zone::Graveyard && o.owner == item.controller)
                        .map(|(id, _)| id as CardId)
                        .collect();
                    for id in ids {
                        self.move_object(id, Zone::Library);
                    }
                    self.forget_library_top(item.controller);
                    self.state
                        .rng
                        .shuffle(&mut self.state.players[item.controller.index()].library);
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
                    if !self.defer_draw_protection(ids.clone(), item.object) {
                        self.grant_keyword_effects(&ids, item.object, Keywords::INDESTRUCTIBLE);
                    }
                }
                Instruction::ShuffleTargetIntoLibrary => self.shuffle_target_into_library(&item),
                Instruction::MultiplyTargetStats { factor } => {
                    self.multiply_target_stats(&item, factor)
                }
                Instruction::OwnerLibrary => {
                    if self.start_target_library(&item, false) {
                        return;
                    }
                }
                Instruction::OwnerLibrarySecondFromTop => {
                    if self.start_target_library(&item, true) {
                        return;
                    }
                }
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
                    }) && self.on_battlefield(item.card)
                        && self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].controller == item.controller
                    {
                        self.sacrifice_batch(&[(item.controller, item.card)]);
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
                Instruction::Modal(_)
                | Instruction::ModalOnce(_)
                | Instruction::ModalOncePerTurn(_) => {
                    unreachable!("unselected modal ability executed")
                }
                Instruction::Explore(subject) => {
                    if self.start_explore(&item, subject) {
                        return;
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
                Instruction::IfKicked { skip } => {
                    if !item.cast.is_some_and(|c| c.kicker == Some(true)) {
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
                    if !self.step_guard(guard, item.controller, item.object, item.source_tapped) {
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
                    for player in self.program_players(set, &item) {
                        let counters = &mut self.state.players[player.index()].poison_counters;
                        *counters = counters
                            .checked_add(u64::from(amount))
                            .capacity("player poison counters exhausted");
                    }
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
                Instruction::BlinkImmediately { counter, tapped } => {
                    self.blink_immediately(&item, counter, tapped)
                }
                Instruction::IfCastFromNonHand { skip } => {
                    if item.is_copy || !item.cast.is_some_and(|cast| cast.origin != Zone::Hand) {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
                }
                Instruction::BlinkUntilEndStep { under_owner } => {
                    self.blink_until_end_step(&item, under_owner)
                }
                Instruction::ReturnDeathCard(kind) => self.return_death_card(&item, kind),
                Instruction::ReturnEventCardToHand => self.return_event_card_to_hand(&item),
                Instruction::ResolveDelayedReturn => self.resolve_delayed_return(&item),
                Instruction::ReturnSourceFromFirstPublicZoneToHand => {
                    self.return_source_from_first_public_zone_to_hand(&item);
                }
                Instruction::ReturnSourceToHand(origin) => {
                    let zone = match origin {
                        TargetZone::Battlefield => Zone::Battlefield,
                        TargetZone::Graveyard => Zone::Graveyard,
                    };
                    if self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].zone == zone
                    {
                        self.move_object(item.card, Zone::Hand);
                    }
                }
                Instruction::PutSourceOnLibraryTop { origin } => {
                    let zone = match origin {
                        TargetZone::Battlefield => Zone::Battlefield,
                        TargetZone::Graveyard => Zone::Graveyard,
                    };
                    if self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].zone == zone
                    {
                        let owner = self.state.objects[item.card as usize].owner;
                        let event = self.move_object(item.card, Zone::Library);
                        if event.moved && event.destination == Zone::Library {
                            self.remember_library_top(owner, event.after);
                        }
                    }
                }
                Instruction::ReturnSourceWithCounterAndEquipment(counter) => {
                    self.return_source_with_equipment(&item, counter);
                }
                Instruction::ReturnSourceToBattlefield { tapped } => {
                    if self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].zone == Zone::Graveyard
                    {
                        let movement = self.entry_move(item.card, Zone::Battlefield, tapped);
                        self.stage_entries(vec![movement], None);
                    }
                }
                Instruction::CounterRelated => {
                    let link = item.related_stack.expect("ward stack association");
                    self.counter_stack_item(link.id, false);
                }
                Instruction::MoveBattlefield { filters, exile } => {
                    self.move_battlefield_group(&item, filters, exile);
                }
                Instruction::GainControlUntilEndTurn => self.gain_control_until_end_turn(&item),
                Instruction::GainControl {
                    subject,
                    controller,
                } => {
                    self.gain_control(&item, subject, controller);
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
                Instruction::CastTargetedGraveyardFree => {
                    if self.offer_targeted_graveyard_free_cast(&item) {
                        return;
                    }
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
                Instruction::Mill(players, amount) => {
                    let players: Vec<_> = self.program_players(players, &item).collect();
                    self.mill_batch(&players, amount, item.controller);
                }
                Instruction::MayMill(amount) => {
                    // CR701.17b forbids choosing to mill more than the library.
                    let resolution = self.state.resolving.as_mut().unwrap();
                    if self.state.players[item.controller.index()].library.len()
                        < usize::from(amount)
                    {
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
                            object.zone == Zone::Graveyard && object.owner == item.controller
                        })
                        .map(|(id, object)| ObjectRef {
                            card: id as CardId,
                            generation: object.generation,
                        })
                        .collect();
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
                        .filter(|(_, object)| object.zone == Zone::Hand && object.owner == victim)
                        .map(|(id, _)| self.object_ref(id as CardId))
                        .collect();
                    for reference in &hand {
                        if !self.state.known_hand.contains(reference) {
                            self.state.known_hand.push(*reference);
                        }
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
                                Some(self.state.objects[id as usize].card.fixed_mana_value())
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
                            Target::Card(id) => {
                                !self.keywords(id).contains(Keywords::INDESTRUCTIBLE)
                            }
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
                Instruction::AttachSource => {
                    if let Some(target) = item.targets.iter().find_map(|bound| bound.object) {
                        self.attach_equipment(item.object, target);
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
                Instruction::CopyControlledNontokenCreaturesNonlegendary => {
                    self.copy_controlled_nontoken_creatures_nonlegendary(item.controller);
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
                Instruction::SacrificeSource => self.sacrifice_exact_source(&item),
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
                Instruction::DistributeCounters { kind, .. } => {
                    self.distribute_counters(&item, kind)
                }
                Instruction::DamageByGroup { first, other } => {
                    self.damage_by_group(&item, first, other)
                }
                Instruction::DamageForExcess(amount) => self.damage_for_excess(&item, amount),
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
                Instruction::TransformSource => self.transform_source(&item),
                Instruction::RequireBlocks(filter) => self.require_blocks(&item, filter),
                Instruction::ActivationX | Instruction::SourceCounterCost { .. } => (),
                Instruction::RemoveCreatureCounterThenDraw => {
                    if self.start_resolution_counter_payment(&item) {
                        return;
                    }
                }
                Instruction::DestroyCombatDamagedPlayersPermanents => {
                    self.destroy_combat_damaged_players_permanents(&item)
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
                Instruction::SelectGroup(group) => {
                    self.state.resolving.as_mut().unwrap().target_group = group;
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
                    self.add_mana(item.controller, [0, 0, 0, 0, 0, amount]);
                }
                Instruction::AddManaScaled { color, amount } => {
                    let amount = u32::try_from(self.item_amount(amount, &item))
                        .capacity("mana representation exhausted");
                    let mut mana = [0; 6];
                    mana[color.index()] = amount;
                    self.add_mana(item.controller, mana);
                }
                Instruction::AddMana(production) => {
                    let quality = self
                        .mana_quality(production, item.card)
                        .map(|quality| quality.with_definition(item.source_card));
                    match production.base().in_world(self, item.card) {
                        ManaProduction::ChosenColor
                        | ManaProduction::MonocoloredPermanentColors
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
                Instruction::AnimateSource(spec) => self.animate_source(&item, spec),
                Instruction::AnimateTargets(spec) => self.animate_targets(&item, spec),
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
                Instruction::Search(spec)
                | Instruction::SearchTargetController(spec)
                | Instruction::SearchTargetPlayer(spec)
                | Instruction::SearchRequired(spec)
                | Instruction::SearchDifferentNames(spec) => {
                    let player = if matches!(instruction, Instruction::SearchTargetController(_)) {
                        let Some(player) = self.resolution_target_controller() else {
                            continue;
                        };
                        player
                    } else if matches!(instruction, Instruction::SearchTargetPlayer(_)) {
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
                        usize::from(spec.maximum)
                            .min(self.state.players[player.index()].library.len())
                            as u16
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
                            ),
                            selected: Vec::new(),
                            battlefield: None,
                            destination_pending: false,
                        }));
                    self.state.pending = Pending::EffectSearch { player };
                    return;
                }
                Instruction::RevealTopDrawIfArtifact => self.reveal_top_draw_if_artifact(&item),
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
                Instruction::RandomLinkedExileToBattlefield => {
                    self.random_linked_exile_to_battlefield(item.object, item.controller)
                }
                Instruction::ExileTopPlayScaled { amount, next_turn } => {
                    let count = self.item_amount(amount, &item);
                    self.exile_top_play(item.controller, count, next_turn);
                }
                Instruction::ExileTopPlay { amount, next_turn } => {
                    self.exile_top_play(item.controller, u128::from(amount), next_turn);
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
                        let amount = library
                            .iter()
                            .rev()
                            .position(|&id| self.state.objects[id as usize].card.is_land())
                            .map_or(library.len(), |n| n + 1);
                        let ids = library
                            .iter()
                            .rev()
                            .take(amount)
                            .copied()
                            .collect::<Vec<_>>();
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
                        && self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].zone == Zone::Stack
                    {
                        self.move_object(item.card, Zone::Exile);
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
                        .take(usize::from(amount))
                        .copied()
                        .collect();
                    if !remaining.is_empty() {
                        self.state.resolving.as_mut().unwrap().library = Some(LibraryChoice {
                            surveil: matches!(instruction, Instruction::Surveil(_)),
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
                Instruction::PayXReflexive { .. } => {
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
                        if self.on_battlefield(item.card)
                            && self.object_ref(item.card) == item.object
                        {
                            self.state.pending = Pending::Endure {
                                player: item.controller,
                                source: item.object,
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
                            source: item.object,
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
                Instruction::RemoveSourceCounterIfAny { kind, skip } => {
                    if self.on_battlefield(item.card)
                        && self.object_ref(item.card) == item.object
                        && self.counter_count(item.card, kind) > 0
                    {
                        self.pay_counters(item.card, &Counters::of(kind, 1));
                    } else {
                        self.state.resolving.as_mut().unwrap().pc += skip;
                    }
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
                        replacements::TokenCompletion::JobAttach(item.object),
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
                        replacements::TokenCompletion::AttachCreatedTo(item.object),
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
                Instruction::ExileGraveyard(set) => {
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    let ids: Vec<_> = self
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.zone == Zone::Graveyard && players.contains(&object.owner)
                        })
                        .map(|(id, _)| id as CardId)
                        .collect();
                    let events = self.move_objects_raw(&ids, Zone::Exile);
                    self.finish_zone_events(&events, &[]);
                }
                Instruction::AddCastEntryCounters { kind, amount } => {
                    self.add_cast_entry_counters(&item, kind, amount)
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
                Instruction::EndTurn => {
                    self.end_turn_effect();
                    return;
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
                Instruction::LoseLife(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.lose_life(player, i128::from(amount));
                    }
                }
                Instruction::UntapPlayerCreatures(players) => {
                    let recipients = self.player_creature_recipients(players, &item);
                    self.untap_objects(&recipients, untap_rules::UntapCause::Effect);
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
                                item.object,
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
                    if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                        if instruction == Instruction::TapSource {
                            self.tap_objects(&[item.card]);
                        } else {
                            self.untap_objects(&[item.card], untap_rules::UntapCause::Effect);
                        }
                    }
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
                } => {
                    let amount = if let Instruction::GrowScaled { amount, .. } = instruction {
                        i128::try_from(self.item_amount(amount, &item))
                            .capacity("growth amount representation exhausted")
                    } else {
                        1
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
                            if self.on_battlefield(item.card)
                                && self.object_ref(item.card) == item.object
                            {
                                self.state.objects[item.card as usize].growth += stats;
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
                    if self.on_battlefield(item.object.card)
                        && self.object_ref(item.object.card) == item.object
                    {
                        let event = self.move_object(item.object.card, Zone::Exile);
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
                                && self.creature_matches(filter, item.controller, item.object, id)
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
                Instruction::GrantKeywords { subject, keywords } => {
                    let ids = self.keyword_subjects(&item, subject);
                    self.grant_keyword_effects(&ids, item.object, keywords);
                }
                Instruction::GrantCreatures { filter, keywords } => {
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(filter, item.controller, item.object, id)
                        })
                        .collect();
                    self.grant_keyword_effects(&affected, item.object, keywords);
                }
                Instruction::SnapshotDoubleCreatures(filter) => {
                    // Snapshot every operand before changing any derived characteristic.
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(filter, item.controller, item.object, id)
                        })
                        .map(|id| (id, self.stats(id)))
                        .collect();
                    for (id, delta) in affected {
                        self.state.objects[id as usize].growth += delta;
                    }
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
                            self.creature_matches(filter, item.controller, item.object, id)
                                && excluding_subtype
                                    .is_none_or(|subtype| !self.has_subtype(id, subtype))
                        })
                        .collect();
                    for id in affected {
                        self.state.objects[id as usize].growth += delta;
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
                            self.creature_matches(filter, item.controller, item.object, id)
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
                    if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                        self.effect_counter(
                            item.card,
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
                                item.object,
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
                } => {
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(filter, item.controller, item.object, id)
                        })
                        .map(|id| {
                            let mut c = Counters::default();
                            c.add(kind, u64::from(amount));
                            (self.object_ref(id), c)
                        })
                        .collect();
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
                Instruction::RemoveAllCounters => {
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            self.set_counters(id, Counters::default());
                        }
                    }
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
                            if self.on_battlefield(item.card)
                                && self.object_ref(item.card) == item.object
                            {
                                vec![item.card]
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
                                Target::Card(id) => {
                                    !self.keywords(id).contains(Keywords::INDESTRUCTIBLE)
                                }
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
