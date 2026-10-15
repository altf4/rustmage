//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub item: SpellView,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub target_group: u16,
    pub discards: [u16; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub library: Option<LibraryChoice>,
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
        library.truncate(library.len() - count);
        // Reuse library capacity; only bottom insertion shifts the untouched middle.
        library.splice(0..0, choice.bottom.iter().rev().copied());
        library.extend(choice.top.iter().rev().copied());
        self.run_program();
    }

    pub(super) fn next_stack_id(&mut self) -> u64 {
        let id = self.state.next_stack_id;
        self.state.next_stack_id = id.checked_add(1).expect("stack identity exhausted");
        id
    }

    pub(super) fn gain_life(&mut self, player: Player, amount: i64) {
        if amount <= 0 {
            return;
        }
        let state = &mut self.state.players[player.index()];
        state.life = state
            .life
            .checked_add(amount)
            .expect("life total exhausted");
        state.life_gain_events_this_turn = state
            .life_gain_events_this_turn
            .checked_add(1)
            .expect("life gain event count exhausted");
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
                } = ability.condition
                    && (!first_only || first)
                    && (!own_turn || self.state.active == player)
                {
                    self.queue_trigger(&source, index);
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

    pub(super) fn zone_triggers(&mut self, event: &ZoneChange, death_sources: &[ObjectView]) {
        if !event.moved {
            return;
        }
        if event.before.zone == Zone::Battlefield
            && event.destination == Zone::Graveyard
            && event.before.card.is_creature()
        {
            // Dies triggers look back at the entire battlefield before the batch;
            // a source that dies simultaneously still sees the other creatures.
            for source in death_sources {
                for (index, ability) in source.card.triggers().iter().enumerate() {
                    let matched = match ability.condition {
                        TriggerCondition::SelfDies => source.reference == event.before.reference,
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
                            related
                                && (!other || source.reference != event.before.reference)
                                && (!nontoken || event.before.token.is_none())
                                && subtype.is_none_or(|subtype| {
                                    event
                                        .before
                                        .card
                                        .info()
                                        .type_line
                                        .split_once(" — ")
                                        .is_some_and(|(_, types)| {
                                            types.split_whitespace().any(|word| word == subtype)
                                        })
                                })
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
        // Enter triggers see the post-event battlefield, including the newcomer.
        let sources: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, obj)| obj.zone == Zone::Battlefield && !obj.card.triggers().is_empty())
            .map(|(i, _)| self.object_view(i as CardId))
            .collect();
        let entered = self.object_view(event.after.card);
        for source in &sources {
            let same = source.reference == event.after;
            let yours = source.owner == entered.owner;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition {
                    TriggerCondition::SelfEnters | TriggerCondition::SelfEntersOrAttacks => same,
                    TriggerCondition::SelfDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SpellCast { .. }
                    | TriggerCondition::AtStep { .. } => false,
                    TriggerCondition::OtherCreatureEntersYou => {
                        yours && !same && entered.card.is_creature()
                    }
                    TriggerCondition::CreatureEntersYou => yours && entered.card.is_creature(),
                    TriggerCondition::LandEntersYou => yours && entered.card.is_land(),
                    TriggerCondition::OtherArtifactEntersYou => {
                        yours && !same && entered.card.is_artifact()
                    }
                };
                if matched {
                    self.queue_trigger(source, index);
                }
            }
        }
    }

    pub(super) fn queue_trigger(&mut self, source: &ObjectView, index: usize) {
        let id = self.next_stack_id();
        self.state.pending_triggers.push(SpellView {
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_card: source.card,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            card: source.id,
            object: source.reference,
            controller: source.owner, // Control-changing effects remain gated.
            targets: Vec::new(),
            mode: None,
        });
    }

    /// A spell becomes cast only after every declaration and cost is complete.
    /// Completed insertion paths share this hook; abilities never count as casts.
    pub(super) fn push_stack_item(&mut self, item: SpellView) {
        let cast = (item.kind == StackKind::Spell).then_some((item.controller, item.source_card));
        self.state.stack.push(item);
        let Some((caster, card)) = cast else { return };
        self.record_spell_cast(caster, card);
    }

    pub(super) fn record_spell_cast(&mut self, caster: Player, card: Card) {
        let count = &mut self.state.players[caster.index()].spells_cast_this_turn;
        *count = count.checked_add(1).expect("spell count exhausted");
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
                } = ability.condition
                {
                    let related = match relation {
                        CastRelation::You => source.owner == caster,
                        CastRelation::Opponent => source.owner != caster,
                        CastRelation::Any => true,
                    };
                    if related
                        && filter.matches(card)
                        && wanted.is_none_or(|wanted| wanted == ordinal)
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
                ability.condition,
                TriggerCondition::SelfAttacks | TriggerCondition::SelfEntersOrAttacks
            ) {
                self.queue_trigger(source, index);
            }
        }
    }

    pub(super) fn prepare_triggers(&mut self) {
        if self.state.pending != Pending::Priority
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
            && let TriggerCondition::AtStep { guard, .. } =
                item.source_card.triggers()[ability as usize].condition
            && !self.step_guard(guard, item.controller, item.object, item.source_tapped)
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
            StackKind::TriggeredAbility { ability } => {
                item.source_card.triggers()[ability as usize].program
            }
            StackKind::ActivatedAbility { ability } => {
                item.source_card.activations()[ability as usize].program
            }
            StackKind::Spell => match item.source_card.effect() {
                Effect::Creature(..) | Effect::Permanent => {
                    self.move_object(item.card, Zone::Battlefield);
                    self.give_priority();
                    return;
                }
                Effect::Spell(..) | Effect::Modal(_) => {
                    item.source_card.spell_program(item.mode).unwrap().1
                }
                Effect::Land | Effect::Unsupported => unreachable!("unsupported spell admitted"),
            },
        };
        assert!(self.state.resolving.is_none());
        self.state.resolving = Some(Resolution {
            item,
            program,
            pc: 0,
            target_group: 0,
            discards: [0; 2],
            selected_discards: [Vec::new(), Vec::new()],
            library: None,
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
            for player in [self.state.active, self.state.active.other()] {
                for &card in &selected[player.index()] {
                    self.move_object(card, Zone::Graveyard);
                }
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
                Instruction::CreateTokens {
                    token,
                    amount,
                    tapped,
                } => {
                    self.create_tokens(item.controller, token, amount, tapped);
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
                Instruction::Draw(set, amount) => {
                    for player in self.program_players(set, &item) {
                        for _ in 0..amount {
                            self.draw(player);
                        }
                    }
                }
                Instruction::Discard(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.state.resolving.as_mut().unwrap().discards[player.index()] = amount;
                    }
                }
                Instruction::GainLife(set, amount) => {
                    let players: Vec<_> = self.program_players(set, &item).collect();
                    for player in players {
                        self.gain_life(player, i64::from(amount));
                    }
                }
                Instruction::LoseLife(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.state.players[player.index()].life -= i64::from(amount);
                    }
                }
                Instruction::UntapCreatures(filter) => {
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
                        .map(|(id, _)| id)
                        .collect();
                    for id in ids {
                        self.state.objects[id].untap();
                    }
                }
                Instruction::UntapSource => {
                    if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                        self.state.objects[item.card as usize].untap();
                    }
                }
                Instruction::Damage(amount) | Instruction::DamagePlayers(_, amount) => {
                    let source = if self.object_ref(item.card) == item.object {
                        self.damage_source(item.card)
                    } else {
                        DamageSource {
                            object: item.card,
                            reference: item.object,
                            controller: item.controller,
                            keywords: item.source_keywords,
                        }
                    };
                    let event = |target| DamageEvent {
                        source,
                        target,
                        amount,
                        combat: false,
                    };
                    let events: Vec<_> = if let Instruction::DamagePlayers(players, _) = instruction
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
                } => {
                    let stats = Stats { power, toughness };
                    match subject {
                        Subject::Source => {
                            if self.on_battlefield(item.card)
                                && self.object_ref(item.card) == item.object
                            {
                                self.state.objects[item.card as usize].growth += stats;
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
                        self.state.objects[id as usize].growth += Stats { power, toughness };
                    }
                }
                Instruction::PutCounters {
                    subject,
                    kind,
                    amount,
                } => match subject {
                    Subject::Source => {
                        if self.on_battlefield(item.card)
                            && self.object_ref(item.card) == item.object
                        {
                            self.state.objects[item.card as usize]
                                .counters
                                .add(kind, amount);
                        }
                    }
                    Subject::Targets => {
                        for bound in &item.targets {
                            if self.same_target_object(*bound)
                                && let Target::Card(id) = bound.target
                            {
                                self.state.objects[id as usize].counters.add(kind, amount);
                            }
                        }
                    }
                },
                Instruction::Tap | Instruction::Untap => {
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            let object = &mut self.state.objects[id as usize];
                            if instruction == Instruction::Tap {
                                object.tapped = true;
                            } else {
                                object.untap();
                            }
                        }
                    }
                }
                Instruction::Counter
                | Instruction::Exile
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
                                Target::Card(id) => !self.state.objects[id as usize]
                                    .keywords()
                                    .contains(Keywords::INDESTRUCTIBLE),
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
