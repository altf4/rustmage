//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;
use crate::cards::{CastFilter, CastTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub connive: Option<Box<super::connive::ConniveSequence>>,
    pub item: SpellView,
    pub mana_payment: Option<Box<resolution_mana::ResolutionPayment>>,
    pub forage: Option<Box<forage::ForageSelection>>,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub target_group: u16,
    pub moved_count: Option<u64>,
    pub discards: [usize; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub milled: Vec<ObjectRef>,
    pub library: Option<LibraryChoice>,
    pub selection: Option<Box<super::library_selection::LibrarySelection>>,
    pub search: Option<super::search::SearchChoice>,
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

    pub(super) fn gain_life(&mut self, player: Player, amount: i128) {
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
            if source.owner != player {
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
        if event.before.zone == Zone::Battlefield && event.destination == Zone::Graveyard {
            // Departure triggers look back at the entire pre-event battlefield;
            // simultaneous departures still see each other exactly once.
            for source in death_sources {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    let matched = match ability.condition.event() {
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
                                CastRelation::You => source.owner == event.before.owner,
                                CastRelation::Opponent => source.owner != event.before.owner,
                                CastRelation::Any => true,
                            };
                            event.before.card.is_creature()
                                && related
                                && (!other || source.reference != event.before.reference)
                                && (!nontoken || event.before.token.is_none())
                                && subtype
                                    .is_none_or(|subtype| event.before.card.has_subtype(subtype))
                        }
                        TriggerCondition::Sacrificed {
                            filter: f,
                            own_turn,
                        } => {
                            event.sacrificed_by == Some(source.owner)
                                && (!own_turn || self.state.active == source.owner)
                                && (!f.source_only || source.reference == event.before.reference)
                                && (!f.other || source.reference != event.before.reference)
                                && (f.kinds == 0
                                    || event.before.card.kind_mask() & u16::from(f.kinds) != 0)
                                && (!f.nonland || !event.before.card.is_land())
                                && event.before.keywords.contains(f.keywords)
                                && f.tokens
                                    .is_none_or(|token| event.before.card.is_token() == token)
                                && f.subtype
                                    .is_none_or(|subtype| event.before.card.has_subtype(subtype))
                        }
                        TriggerCondition::CreatureDiesFiltered { filter } => {
                            event.before.card.is_creature()
                                && filter.matches_intrinsic(
                                    event.before.card,
                                    event.before.owner == source.owner,
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
                                CastRelation::You => source.owner == event.before.owner,
                                CastRelation::Opponent => source.owner != event.before.owner,
                                CastRelation::Any => true,
                            };
                            event.before.card.kind_mask() & kinds != 0
                                && related
                                && (source.reference != event.before.reference
                                    || event.before.card.kind_mask() & exclude_source_kinds == 0)
                        }
                        _ => false,
                    };
                    if matched {
                        self.queue_trigger(source, index);
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
        if entered.card.kind_mask() & entry_suppression != 0 {
            return;
        }
        for source in sources {
            let same = source.reference == event.after;
            let yours = source.owner == entered.owner;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::OncePerTurn(_)
                    | TriggerCondition::DuringOpponentsTurn(_)
                    | TriggerCondition::InterveningIf { .. } => {
                        unreachable!("unwrapped event")
                    }
                    TriggerCondition::SelfOffspring => same && event.offspring_paid,
                    TriggerCondition::SelfEnters
                    | TriggerCondition::SelfEntersOrAttacks
                    | TriggerCondition::SelfEntersOrDies => same,
                    TriggerCondition::Foraged
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Reflexive
                    | TriggerCondition::Ward
                    | TriggerCondition::SelfTargeted { .. }
                    | TriggerCondition::CardDrawn { .. }
                    | TriggerCondition::SelfDiscarded
                    | TriggerCondition::SelfCycled
                    | TriggerCondition::Discarded { .. }
                    | TriggerCondition::PlayerDamaged { .. }
                    | TriggerCondition::SelfDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::Sacrificed { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::AttackDeclared { .. }
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SelfBecomesTapped
                    | TriggerCondition::SpellCast { .. }
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
                                (kinds == 0 || entered.card.kind_mask() & kinds != 0)
                                    && (!nontoken || entered.token.is_none())
                                    && subtype.is_none_or(|t| entered.card.has_subtype(t))
                            }
                    }
                    TriggerCondition::OtherFilteredEntersYou(predicate) => {
                        yours
                            && !same
                            && match predicate {
                                EntryPredicate::CreaturePowerAtMost(maximum) => {
                                    entered.card.is_creature()
                                        && entered.power <= i128::from(maximum)
                                }
                                EntryPredicate::SubtypeAny(subtypes) => subtypes
                                    .iter()
                                    .any(|subtype| entered.card.has_subtype(subtype)),
                                EntryPredicate::KindOrSubtype { kinds, subtype } => {
                                    entered.card.kind_mask() & kinds != 0
                                        || entered.card.has_subtype(subtype)
                                }
                            }
                    }
                    TriggerCondition::OtherCreatureEntersYou => {
                        yours && !same && entered.card.is_creature()
                    }
                    TriggerCondition::CreatureEnters { relation, other } => {
                        entered.card.is_creature()
                            && (!other || !same)
                            && match relation {
                                CastRelation::You => yours,
                                CastRelation::Opponent => !yours,
                                CastRelation::Any => true,
                            }
                    }
                    TriggerCondition::OtherSubtypeEntersYou(subtype) => {
                        yours && !same && entered.card.has_subtype(subtype)
                    }
                    TriggerCondition::SubtypeEntersYou(subtype) => {
                        yours && entered.card.has_subtype(subtype)
                    }
                    TriggerCondition::ArtifactEntersYou => yours && entered.card.is_artifact(),
                    TriggerCondition::EnchantmentEntersYou => {
                        yours && entered.card.kind_mask() & 32 != 0
                    }
                    TriggerCondition::CreatureEntersYou => yours && entered.card.is_creature(),
                    TriggerCondition::LandEntersYou => yours && entered.card.is_land(),
                    TriggerCondition::OtherArtifactEntersYou => {
                        yours && !same && entered.card.is_artifact()
                    }
                };
                if matched && self.queue_trigger(source, index) {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .entering_object = Some(event.after);
                }
            }
        }
    }

    pub(super) fn queue_trigger(&mut self, source: &ObjectView, index: usize) -> bool {
        let condition = source.card.triggers()[index].condition;
        if !self.trigger_guards_hold(condition, source.owner, source.reference, source.tapped) {
            return false;
        }
        if condition.once_per_turn() {
            let used = TriggerUse {
                source: source.reference,
                ability: u16::try_from(index).expect("ability index"),
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
            cast: None,
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_card: source.card,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            source_facts: DamageFacts::from_view(source),
            related_stack: None,
            entering_object: None,
            damage_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_attachment: source.attached_to,
            card: source.id,
            object: source.reference,
            controller: source.owner, // Control-changing effects remain gated.
            targets: Vec::new(),
            mode: None,
        });
        true
    }

    /// A spell becomes cast only after every declaration and cost is complete.
    /// Completed insertion paths share this hook; abilities never count as casts.
    pub(super) fn push_stack_item(&mut self, item: SpellView) {
        let cast = (item.kind == StackKind::Spell).then_some((item.controller, item.source_card));
        self.state.stack.push(item);
        let Some((caster, card)) = cast else { return };
        self.record_cast(caster, card, Some(self.state.stack.len() - 1));
    }

    #[cfg(test)]
    pub(super) fn record_spell_cast(&mut self, caster: Player, card: Card) {
        self.record_cast(caster, card, None);
    }

    /// CR115.9b: inspect the current expected zone/incarnation; never use target LKI.
    pub(super) fn cast_filter_matches(
        &self,
        filter: CastFilter,
        card: Card,
        source: &ObjectView,
        targets: &[BoundTarget],
    ) -> bool {
        match filter {
            CastFilter::AnyOf(filters) => filters
                .iter()
                .any(|f| self.cast_filter_matches(*f, card, source, targets)),
            CastFilter::ManaValueMaximum { filter, maximum } => {
                card.fixed_mana_value() <= maximum
                    && self.cast_filter_matches(*filter, card, source, targets)
            }
            CastFilter::ManaValueMinimum { filter, minimum } => {
                card.fixed_mana_value() >= minimum
                    && self.cast_filter_matches(*filter, card, source, targets)
            }
            CastFilter::Targeted { filter, target } => {
                self.cast_filter_matches(*filter, card, source, targets)
                    && targets.iter().any(|bound| {
                        if let Target::Player(player) = bound.target {
                            return target == CastTarget::OpponentOrCreature
                                && player != source.owner;
                        }
                        let (Target::Card(id), Some(reference)) = (bound.target, bound.object)
                        else {
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
                                object.card.is_creature() && object.owner != source.owner
                            }
                            CastTarget::Creature(relation) => {
                                object.card.is_creature()
                                    && match relation {
                                        CastRelation::You => object.owner == source.owner,
                                        CastRelation::Opponent => object.owner != source.owner,
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
        let count = &mut self.state.players[caster.index()].spells_cast_this_turn;
        *count = count.checked_add(1).capacity("spell count exhausted");
        let ordinal = *count;
        let sources: Vec<_> = self
            .state
            .cast_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if let TriggerCondition::SpellCast {
                    relation,
                    filter,
                    ordinal: wanted,
                } = ability.condition.event()
                {
                    let related = match relation {
                        CastRelation::You => source.owner == caster,
                        CastRelation::Opponent => source.owner != caster,
                        CastRelation::Any => true,
                    };
                    if related
                        && (!ability.condition.requires_opponents_turn()
                            || self.state.active != source.owner)
                        && self.cast_filter_matches(
                            filter,
                            card,
                            &source,
                            stack
                                .map(|i| self.state.stack[i].targets.as_slice())
                                .unwrap_or(&[]),
                        )
                        && wanted.is_none_or(|wanted| wanted == ordinal)
                    {
                        self.queue_trigger(&source, index);
                    }
                }
            }
        }
    }

    /// Apply one simultaneous tap event, then collect source-owned triggers.
    /// Entry status and virtual mana funding never use this transition.
    pub(super) fn tap_objects(&mut self, ids: &[CardId]) {
        // Mana taps and single-target effects are the common path. Avoid both
        // allocation and evaluated object views for cards with no tap observer.
        if let &[id] = ids {
            let object = &mut self.state.objects[id as usize];
            if object.zone == Zone::Battlefield && !object.tapped {
                object.tapped = true;
                self.tapped_triggers(id);
            }
            return;
        }
        let mut changed = Vec::new();
        for &id in ids {
            let object = &mut self.state.objects[id as usize];
            if object.zone == Zone::Battlefield && !object.tapped {
                object.tapped = true;
                if object.card.triggers().iter().any(|ability| {
                    matches!(
                        ability.condition.event(),
                        TriggerCondition::SelfBecomesTapped
                    )
                }) {
                    changed.push(id);
                }
            }
        }
        // Every observer sees the complete simultaneous tapped result. No queued
        // effect resolves until the surrounding action or instruction completes.
        for id in changed {
            self.tapped_triggers(id);
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
    pub(super) fn declaration_triggers(&mut self, actor: Player) {
        let sources: Vec<_> = self
            .state
            .attack_sources
            .iter()
            .copied()
            .filter(|&id| self.on_battlefield(id) && self.state.objects[id as usize].owner == actor)
            .map(|id| self.object_view(id))
            .collect();
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
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
                            o.owner == actor
                                && o.zone == Zone::Battlefield
                                && o.card.is_creature()
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

    pub(super) fn attack_triggers(&mut self, source: &ObjectView) {
        for (index, ability) in source.card.triggers().iter().enumerate() {
            if matches!(
                ability.condition.event(),
                TriggerCondition::SelfAttacks | TriggerCondition::SelfEntersOrAttacks
            ) {
                self.queue_trigger(source, index);
            }
        }
    }

    pub(super) fn prepare_triggers(&mut self) {
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
            && !self.trigger_guards_hold(
                item.source_card.triggers()[ability as usize].condition,
                item.controller,
                item.object,
                item.source_tapped,
            )
        {
            self.give_priority();
            return;
        }
        let was_targeted = !item.targets.is_empty();
        let spec = self.target_spec(&item);
        item.targets.retain(|bound| {
            self.same_target_object(*bound)
                && spec.group(bound.group).is_some_and(|group| {
                    self.target_matches(group.rule, item.controller, item.object, bound.target)
                })
        });
        let valid = !was_targeted || !item.targets.is_empty();
        if !valid {
            if item.kind == StackKind::Spell {
                self.move_object(item.card, Zone::Graveyard);
            }
            self.give_priority();
            return;
        }
        let program = match item.kind {
            StackKind::TriggeredAbility { .. } => {
                item.program().expect("selected trigger program").1
            }
            StackKind::ActivatedAbility { ability } => {
                item.source_card.activations()[ability as usize].program
            }
            StackKind::Spell => match item.source_card.effect() {
                Effect::Aura(_) => {
                    let target = item.targets[0].object.unwrap();
                    let mut movement = self.entry_move(
                        item.card,
                        Zone::Battlefield,
                        self.enters_tapped(item.card),
                    );
                    movement.attachment = Some(target);
                    self.stage_entries(vec![movement], None);
                    self.give_priority();
                    return;
                }
                Effect::Creature(..) | Effect::Permanent => {
                    if item.cast.is_some_and(|c| c.offspring == Some(true)) {
                        let mut movement = self.entry_move(
                            item.card,
                            Zone::Battlefield,
                            self.enters_tapped(item.card),
                        );
                        movement.offspring_paid = true;
                        self.stage_entries(vec![movement], None);
                    } else {
                        self.move_object(item.card, Zone::Battlefield);
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
        assert!(self.state.resolving.is_none());
        self.state.resolving = Some(Resolution {
            connive: None,
            item,
            program,
            pc: 0,
            target_group: 0,
            moved_count: None,
            discards: [0; 2],
            mana_payment: None,
            forage: None,
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            library: None,
            selection: None,
            search: None,
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

    pub(super) fn run_program(&mut self) {
        loop {
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
            if !discarded.is_empty() {
                self.discard_batch(&discarded, None);
            }
            let resolution = self.state.resolving.as_mut().unwrap();
            let Some(instruction) = resolution.program.get(resolution.pc).copied() else {
                let item = self.state.resolving.take().unwrap().item;
                if item.kind == StackKind::Spell {
                    self.move_object(item.card, Zone::Graveyard);
                }
                self.give_priority();
                return;
            };
            resolution.pc += 1;
            let mut item = resolution.item.clone();
            if !matches!(
                instruction,
                Instruction::Fight { .. } | Instruction::PowerDamage { .. }
            ) {
                item.targets
                    .retain(|bound| bound.group == resolution.target_group);
            }
            match instruction {
                Instruction::Forage { skip } => {
                    self.start_resolution_forage(skip);
                    return;
                }
                Instruction::Reflexive { ability } => {
                    let mut reflexive = item.clone();
                    reflexive.id = self.next_stack_id();
                    reflexive.kind = StackKind::TriggeredAbility { ability };
                    reflexive.targets.clear();
                    reflexive.mode = None;
                    self.state.pending_triggers.push(reflexive);
                }
                Instruction::Modal(_) => unreachable!("unselected modal trigger executed"),
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
                    let amount = self.amount_value(value, item.object, item.controller);
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
                Instruction::MayOpponent { skip } => {
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    self.state.pending = Pending::EffectOptional {
                        player: item.controller.other(),
                        skip_to,
                    };
                    return;
                }
                Instruction::MayPayOpponentLife { amount, skip } => {
                    let player = item.controller.other();
                    let skip_to = resolution.pc.checked_add(skip).expect("program offset");
                    assert!(skip_to <= resolution.program.len());
                    if amount != 0 && self.state.players[player.index()].life < i128::from(amount) {
                        self.state.resolving.as_mut().unwrap().pc = skip_to;
                    } else {
                        self.state.pending = Pending::EffectOptional { player, skip_to };
                        return;
                    }
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
                Instruction::ReturnSourceToBattlefield { tapped } => {
                    if self.object_ref(item.card) == item.object
                        && self.state.objects[item.card as usize].zone == Zone::Graveyard
                    {
                        let movement = self.entry_move(
                            item.card,
                            Zone::Battlefield,
                            tapped || self.enters_tapped(item.card),
                        );
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
                Instruction::DamageBattlefield {
                    filters,
                    players,
                    amount,
                } => {
                    self.damage_battlefield_group(&item, filters, players, amount);
                }
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
                    if let Some(id) = self.attached_subject(&item) {
                        self.tap_objects(&[id]);
                    }
                }
                Instruction::AttachSource => {
                    if let Some(target) = item.targets.iter().find_map(|bound| bound.object) {
                        self.attach_equipment(item.object, target);
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
                    );
                }
                Instruction::AddMana(production) => match production {
                    ManaProduction::Fixed(mana) => self.add_mana(item.controller, mana),
                    ManaProduction::Choice { .. } => {
                        self.state.pending = Pending::EffectMana {
                            player: item.controller,
                            production,
                        };
                        return;
                    }
                },
                Instruction::Search(spec) => {
                    self.state.resolving.as_mut().unwrap().search = Some(search::SearchChoice {
                        spec,
                        selected: Vec::new(),
                    });
                    self.state.pending = Pending::EffectSearch {
                        player: item.controller,
                    };
                    return;
                }
                Instruction::ExileTopPlay { amount, next_turn } => {
                    self.exile_top_play(item.controller, amount, next_turn);
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
                Instruction::CreateTokensScaled {
                    token,
                    amount,
                    tapped,
                } => {
                    let amount = self.amount_value(amount, item.object, item.controller);
                    self.create_tokens(item.controller, token, amount, tapped);
                }
                Instruction::CreateTokens {
                    token,
                    amount,
                    tapped,
                } => {
                    self.create_tokens(item.controller, token, u128::from(amount), tapped);
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
                Instruction::Draw(set, _) | Instruction::DrawScaled(set, _) => {
                    let amount = match instruction {
                        Instruction::Draw(_, amount) => u128::from(amount),
                        Instruction::DrawScaled(_, value) => {
                            self.amount_value(value, item.object, item.controller)
                        }
                        _ => unreachable!(),
                    };
                    for player in self.program_players(set, &item) {
                        self.draw_many(player, amount);
                    }
                }
                Instruction::DrainLifeScaled(value) => {
                    let amount =
                        i128::try_from(self.amount_value(value, item.object, item.controller))
                            .capacity("life drain amount representation exhausted");
                    self.lose_life(item.controller.other(), amount);
                    self.gain_life(item.controller, amount);
                }
                Instruction::Discard(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] =
                            usize::from(amount);
                    }
                }
                Instruction::CreateEmblem(factory) => self.create_emblem(item.controller, factory),
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
                            i128::try_from(self.amount_value(value, item.object, item.controller))
                                .capacity("life gain amount representation exhausted")
                        }
                        _ => unreachable!(),
                    };
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    for player in players {
                        self.gain_life(player, amount);
                    }
                }
                Instruction::LoseLife(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.lose_life(player, i128::from(amount));
                    }
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
                        for id in ids {
                            self.state.objects[id as usize].untap();
                        }
                    }
                }
                Instruction::UntapSource | Instruction::TapSource => {
                    if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                        if instruction == Instruction::TapSource {
                            self.tap_objects(&[item.card]);
                        } else {
                            self.state.objects[item.card as usize].untap();
                        }
                    }
                }
                Instruction::Damage(_)
                | Instruction::DamageScaled(_)
                | Instruction::DamagePlayers(_, _)
                | Instruction::DamageDiscardAmount(_) => {
                    let amount = match instruction {
                        Instruction::Damage(amount) | Instruction::DamagePlayers(_, amount) => {
                            i128::from(amount)
                        }
                        Instruction::DamageScaled(value) => {
                            i128::try_from(self.amount_value(value, item.object, item.controller))
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
                        source,
                        target,
                        amount,
                        combat: false,
                    };
                    let events: Vec<_> = if let Instruction::DamagePlayers(players, _)
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
                        i128::try_from(self.amount_value(amount, item.object, item.controller))
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
                Instruction::GrantKeywords { subject, keywords } => match subject {
                    Subject::Entering => {
                        if let Some(id) = self.entering_subject(&item) {
                            let object = &mut self.state.objects[id as usize];
                            object.gained_keywords = object.gained_keywords.union(keywords);
                        }
                    }
                    Subject::Attached => {
                        if let Some(id) = self.attached_subject(&item) {
                            let object = &mut self.state.objects[id as usize];
                            object.gained_keywords = object.gained_keywords.union(keywords);
                        }
                    }
                    Subject::Source => {
                        if self.on_battlefield(item.card)
                            && self.object_ref(item.card) == item.object
                        {
                            let object = &mut self.state.objects[item.card as usize];
                            object.gained_keywords = object.gained_keywords.union(keywords);
                        }
                    }
                    Subject::Targets => {
                        for bound in &item.targets {
                            if self.same_target_object(*bound)
                                && let Target::Card(id) = bound.target
                            {
                                let object = &mut self.state.objects[id as usize];
                                object.gained_keywords = object.gained_keywords.union(keywords);
                            }
                        }
                    }
                },
                Instruction::GrantCreatures { filter, keywords } => {
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(filter, item.controller, item.object, id)
                        })
                        .collect();
                    for id in affected {
                        let object = &mut self.state.objects[id as usize];
                        object.gained_keywords = object.gained_keywords.union(keywords);
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
                        self.state.objects[item.card as usize].counters.add(
                            kind,
                            item.discard_event
                                .expect("discard-sized effect requires event")
                                .amount
                                .into(),
                        );
                    }
                }
                Instruction::PutCreatureCounters {
                    filter,
                    kind,
                    amount,
                } => {
                    // One instruction fixes the entire affected set before any counters change.
                    let affected: Vec<_> = (0..self.state.objects.len())
                        .map(|id| id as CardId)
                        .filter(|&id| {
                            self.creature_matches(filter, item.controller, item.object, id)
                        })
                        .collect();
                    for id in affected {
                        self.state.objects[id as usize]
                            .counters
                            .add(kind, u64::from(amount));
                    }
                }
                Instruction::DoubleCounters => {
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            let counters = &mut self.state.objects[id as usize].counters;
                            counters.plus_one = counters
                                .plus_one
                                .checked_mul(2)
                                .capacity("counter representation exhausted");
                            counters.minus_one = counters
                                .minus_one
                                .checked_mul(2)
                                .capacity("counter representation exhausted");
                            counters.stun = counters
                                .stun
                                .checked_mul(2)
                                .capacity("counter representation exhausted");
                            counters.loyalty = counters
                                .loyalty
                                .checked_mul(2)
                                .capacity("counter representation exhausted");
                        }
                    }
                }
                Instruction::PutCounters {
                    subject,
                    kind,
                    amount,
                } => match subject {
                    Subject::Entering => {
                        if let Some(id) = self.entering_subject(&item) {
                            self.state.objects[id as usize]
                                .counters
                                .add(kind, u64::from(amount));
                        }
                    }
                    Subject::Attached => {
                        if let Some(id) = self.attached_subject(&item) {
                            self.state.objects[id as usize]
                                .counters
                                .add(kind, u64::from(amount));
                        }
                    }
                    Subject::Source => {
                        if self.on_battlefield(item.card)
                            && self.object_ref(item.card) == item.object
                        {
                            self.state.objects[item.card as usize]
                                .counters
                                .add(kind, u64::from(amount));
                        }
                    }
                    Subject::Targets => {
                        for bound in &item.targets {
                            if self.same_target_object(*bound)
                                && let Target::Card(id) = bound.target
                            {
                                self.state.objects[id as usize]
                                    .counters
                                    .add(kind, u64::from(amount));
                            }
                        }
                    }
                },
                Instruction::Tap | Instruction::Untap => {
                    if instruction == Instruction::Untap {
                        for bound in &item.targets {
                            if self.same_target_object(*bound)
                                && let Target::Card(id) = bound.target
                            {
                                self.state.objects[id as usize].untap();
                            }
                        }
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
                        if let Target::Spell(card) = bound.target {
                            let victim = self
                                .state
                                .stack
                                .iter()
                                .find(|s| {
                                    s.kind == StackKind::Spell
                                        && s.card == card
                                        && Some(s.object) == bound.object
                                })
                                .map(|s| s.id);
                            if let Some(id) = victim {
                                self.counter_stack_item(id, false);
                            }
                        }
                    }
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
