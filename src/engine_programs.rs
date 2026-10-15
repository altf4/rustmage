//! Typed immutable programs and owned, snapshot-safe execution continuations.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    pub item: SpellView,
    pub program: &'static [Instruction],
    pub pc: usize,
    pub discards: [u16; 2],
    pub selected_discards: [Vec<CardId>; 2],
}

impl Game {
    pub(super) fn next_stack_id(&mut self) -> u64 {
        let id = self.state.next_stack_id;
        self.state.next_stack_id = id.checked_add(1).expect("stack identity exhausted");
        id
    }

    pub(super) fn death_trigger_sources(&self) -> Vec<ObjectView> {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, obj)| {
                obj.zone == Zone::Battlefield
                    && obj
                        .card
                        .triggers()
                        .iter()
                        .any(|ability| ability.condition == TriggerCondition::SelfDies)
            })
            .map(|(i, _)| self.object_view(i as CardId))
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
            for source in death_sources {
                if source.reference == event.before.reference {
                    for (index, ability) in source.card.triggers().iter().enumerate() {
                        if ability.condition == TriggerCondition::SelfDies {
                            self.queue_trigger(source, index);
                        }
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
                    TriggerCondition::SelfDies | TriggerCondition::SelfAttacks => false,
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

    fn queue_trigger(&mut self, source: &ObjectView, index: usize) {
        let id = self.next_stack_id();
        self.state.pending_triggers.push(SpellView {
            id,
            kind: StackKind::TriggeredAbility {
                ability: u16::try_from(index).expect("ability index"),
            },
            source_card: source.card,
            source_keywords: source.keywords,
            card: source.id,
            object: source.reference,
            controller: source.owner, // Control-changing effects remain gated.
            targets: Vec::new(),
        });
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
        let was_targeted = !item.targets.is_empty();
        let spec = self.target_spec(&item);
        item.targets.retain(|bound| {
            self.same_target_object(*bound)
                && self.target_matches(spec.rule, item.controller, item.object, bound.target)
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
                Effect::Spell(_, program) => program,
                Effect::Land | Effect::Unsupported => unreachable!("unsupported spell admitted"),
            },
        };
        assert!(self.state.resolving.is_none());
        self.state.resolving = Some(Resolution {
            item,
            program,
            pc: 0,
            discards: [0; 2],
            selected_discards: [Vec::new(), Vec::new()],
        });
        self.run_program();
    }

    fn program_players(
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
            let item = resolution.item.clone();
            match instruction {
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
                    for player in self.program_players(set, &item) {
                        self.state.players[player.index()].life += i64::from(amount);
                    }
                }
                Instruction::LoseLife(set, amount) => {
                    for player in self.program_players(set, &item) {
                        self.state.players[player.index()].life -= i64::from(amount);
                    }
                }
                Instruction::Damage(amount) => {
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
                    let events: Vec<_> = item
                        .targets
                        .iter()
                        .copied()
                        .filter(|&bound| self.same_target_object(bound))
                        .map(|bound| DamageEvent {
                            source,
                            target: bound.target,
                            amount,
                            combat: false,
                        })
                        .collect();
                    self.apply_damage_batch(&events);
                }
                Instruction::Grow(amount) => {
                    for bound in &item.targets {
                        if self.same_target_object(*bound)
                            && let Target::Card(id) = bound.target
                        {
                            self.state.objects[id as usize].growth += amount;
                        }
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
                | Instruction::Destroy
                | Instruction::ReturnToHand
                | Instruction::ReturnToBattlefield => {
                    let zone = if instruction == Instruction::ReturnToHand {
                        Zone::Hand
                    } else if instruction == Instruction::ReturnToBattlefield {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    };
                    self.move_target_batch(&item.targets, zone);
                }
            }
        }
    }
}
