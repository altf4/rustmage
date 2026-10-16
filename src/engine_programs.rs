//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub connive: Option<Box<super::connive::ConniveSequence>>,
    pub item: SpellView,
    pub mana_payment: Option<Box<resolution_mana::ResolutionPayment>>,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub target_group: u16,
    pub moved_count: Option<u64>,
    pub discards: [usize; 2],
    pub selected_discards: [Vec<CardId>; 2],
    pub milled: Vec<ObjectRef>,
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
    ) {
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
                            related
                                && (!other || source.reference != event.before.reference)
                                && (!nontoken || event.before.token.is_none())
                                && subtype
                                    .is_none_or(|subtype| event.before.card.has_subtype(subtype))
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
        for source in sources {
            let same = source.reference == event.after;
            let yours = source.owner == entered.owner;
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::OncePerTurn(_) => unreachable!("unwrapped event"),
                    TriggerCondition::SelfEnters
                    | TriggerCondition::SelfEntersOrAttacks
                    | TriggerCondition::SelfEntersOrDies => same,
                    TriggerCondition::Ward
                    | TriggerCondition::CardDrawn { .. }
                    | TriggerCondition::SelfDiscarded
                    | TriggerCondition::SelfCycled
                    | TriggerCondition::Discarded { .. }
                    | TriggerCondition::PlayerDamaged { .. }
                    | TriggerCondition::SelfDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::LifeGained { .. }
                    | TriggerCondition::SelfAttacks
                    | TriggerCondition::SpellCast { .. }
                    | TriggerCondition::AtStep { .. } => false,
                    TriggerCondition::OtherCreatureEntersYou => {
                        yours && !same && entered.card.is_creature()
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
                if matched {
                    self.queue_trigger(source, index);
                }
            }
        }
    }

    pub(super) fn queue_trigger(&mut self, source: &ObjectView, index: usize) -> bool {
        if matches!(
            source.card.triggers()[index].condition,
            TriggerCondition::OncePerTurn(_)
        ) {
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
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_card: source.card,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            source_facts: DamageFacts::from_view(source),
            related_stack: None,
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
        self.record_spell_cast(caster, card);
    }

    pub(super) fn record_spell_cast(&mut self, caster: Player, card: Card) {
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
            && let TriggerCondition::AtStep { guard, .. } = item.source_card.triggers()
                [ability as usize]
                .condition
                .event()
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
                    self.move_object(item.card, Zone::Battlefield);
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
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
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
                        self.state.objects[id as usize].tapped = true;
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
                Instruction::Draw(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.draw_many(player, u128::from(amount));
                    }
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
                        let life = &mut self.state.players[player.index()].life;
                        *life = life
                            .checked_sub(amount)
                            .capacity("life representation exhausted");
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
                        let life = &mut self.state.players[player.index()].life;
                        *life = life
                            .checked_sub(i128::from(amount))
                            .capacity("life representation exhausted");
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
                        .map(|(id, _)| id)
                        .collect();
                    for id in ids {
                        if matches!(instruction, Instruction::TapCreatures(_)) {
                            self.state.objects[id].tapped = true;
                        } else {
                            self.state.objects[id].untap();
                        }
                    }
                }
                Instruction::UntapSource | Instruction::TapSource => {
                    if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                        if instruction == Instruction::TapSource {
                            self.state.objects[item.card as usize].tapped = true;
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
                } => {
                    let stats = Stats {
                        power: i128::from(power),
                        toughness: i128::from(toughness),
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
