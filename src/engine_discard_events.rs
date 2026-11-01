//! Cause-specific simultaneous discards and snapshot-owned trigger quantities.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DiscardTriggerView {
    pub player: Player,
    pub amount: u32,
    pub cycling: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DiscardAfter {
    Recruit(Player, Card),
    LoseLife(Player, i128),
    DrawActual { players: [bool; 2], bonus: u16 },
    RememberActual,
    DrawUnlessOpponentDiscardedTwoNonland(Player),
    NonlandReflexive { controller: Player, ability: u16 },
    ConniveCounters(ObjectRef, u64, Player),
    ChooseLand { player: Player, tapped: bool },
    ReturnLandFromGraveyard { player: Player, tapped: bool },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscardContext {
    sources: Vec<ObjectView>,
    cycled: Option<ObjectRef>,
    cycling_x: Option<u32>,
    shuffles: Vec<ObjectRef>,
    pub after: Option<DiscardAfter>,
}
impl DiscardContext {
    /// Heap-owned initialized elements; the entry batch owns the inline context.
    pub fn owned_bytes(&self) -> usize {
        self.sources.len() * std::mem::size_of::<ObjectView>()
            + self
                .sources
                .iter()
                .map(ObjectView::owned_bytes)
                .sum::<usize>()
            + self.shuffles.len() * std::mem::size_of::<ObjectRef>()
    }
}

impl Game {
    pub(super) fn discard_batch(&mut self, cards: &[CardId], cycled: Option<ObjectRef>) {
        self.discard_batch_caused(cards, cycled, None);
    }
    pub(super) fn discard_batch_caused(
        &mut self,
        cards: &[CardId],
        cycled: Option<ObjectRef>,
        cause: Option<Player>,
    ) {
        self.discard_batch_with_after(cards, cycled, cause, None);
    }
    pub(super) fn discard_batch_with_after(
        &mut self,
        cards: &[CardId],
        cycled: Option<ObjectRef>,
        cause: Option<Player>,
        after: Option<DiscardAfter>,
    ) {
        self.discard_batch_with_cycle_x(cards, cycled, cause, after, None);
    }
    pub(super) fn discard_cycling(&mut self, card: ObjectRef, x: Option<u32>) {
        self.discard_batch_with_cycle_x(&[card.card], Some(card), None, None, x);
    }
    fn discard_batch_with_cycle_x(
        &mut self,
        cards: &[CardId],
        cycled: Option<ObjectRef>,
        cause: Option<Player>,
        after: Option<DiscardAfter>,
        cycling_x: Option<u32>,
    ) {
        // Discard moves cards from hand; transient copies in the hand are not
        // eligible even when an effect supplies a broad object list.
        let cards: Vec<_> = cards
            .iter()
            .copied()
            .filter(|&id| self.state.objects[id as usize].is_card())
            .collect();
        if cards.is_empty() {
            self.finish_discard_after(after, [0; 2], &[]);
            return;
        }
        assert!(
            cards
                .iter()
                .all(|&id| self.state.objects[id as usize].zone == Zone::Hand)
        );
        let sources: Vec<_> = self
            .state
            .discard_sources
            .iter()
            .map(|&id| self.object_view(id))
            .collect();
        let context = DiscardContext {
            sources,
            cycled,
            cycling_x,
            shuffles: Vec::new(),
            after,
        };
        let redirected_entry = cards.iter().any(|&id| {
            let o = &self.state.objects[id as usize];
            o.card.zone_replacement()
                == Some(crate::cards::ZoneReplacementRule::OpponentDiscardToBattlefield)
                && cause.is_some_and(|source| source != o.owner)
        });
        if redirected_entry {
            let frozen: Vec<_> = cards
                .iter()
                .map(|&id| {
                    let before = self.object_view(id);
                    (
                        self.freeze_zone_rules(&before, Zone::Graveyard, cause),
                        before,
                    )
                })
                .collect();
            let mut moves = Vec::new();
            let mut context = context;
            for (rules, before) in frozen {
                let plan = self.plan_zone_move(&before, Zone::Graveyard, rules);
                let mut movement = self.entry_move(before.id, plan.destination, false);
                if plan.shuffle {
                    context.shuffles.push(before.reference);
                }
                movement.controller = before.owner;
                movement.before = before;
                moves.push(movement);
            }
            self.stage_discard_entries(moves, context);
            return;
        }
        let events = self.move_objects_raw(&cards, Zone::Graveyard);
        self.finish_discard_events(&events, context);
    }
    pub(super) fn finish_discard_events(&mut self, events: &[ZoneChange], context: DiscardContext) {
        let DiscardContext {
            sources,
            cycled,
            cycling_x,
            shuffles,
            after,
        } = context;
        for event in events {
            if event.destination == Zone::Library && shuffles.contains(&event.before.reference) {
                self.reveal_and_shuffle_zone_card(&event.before);
            }
        }
        let mut counts = [0u32; 2];
        for event in events.iter().filter(|event| event.moved) {
            let player = event.before.owner;
            counts[player.index()] = counts[player.index()]
                .checked_add(1)
                .capacity("discard count exhausted");
        }
        // Commit the entire simultaneous batch before any discard/zone observer.
        for (p, count) in self.state.players.iter_mut().zip(counts) {
            p.cards_discarded_this_turn = p
                .cards_discarded_this_turn
                .checked_add(u64::from(count))
                .capacity("cards discarded this turn representation exhausted");
        }
        self.record_mayhem_discards(events);
        self.finish_zone_events(events, &[]);
        for event in events.iter().filter(|event| event.moved) {
            let player = event.before.owner;
            let cycling = cycled == Some(event.before.reference);
            // Self-discard/cycle triggers use the card in its destination zone.
            let source = self.object_view(event.after.card);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::SelfDiscarded
                    || cycling && ability.condition.event() == TriggerCondition::SelfCycled
                {
                    let before = self.state.pending_triggers.len();
                    self.queue_discard_trigger(
                        &source,
                        index,
                        DiscardTriggerView {
                            player,
                            amount: 1,
                            cycling,
                        },
                    );
                    if cycling
                        && ability.condition.event() == TriggerCondition::SelfCycled
                        && self.state.pending_triggers.len() > before
                    {
                        self.state.pending_triggers.last_mut().unwrap().x = cycling_x;
                    }
                }
            }
        }
        for source in sources {
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let TriggerCondition::Discarded {
                    relation,
                    one_or_more,
                } = ability.condition.event()
                else {
                    continue;
                };
                for player in [self.state.active, self.state.active.other()] {
                    let count = counts[player.index()];
                    if count == 0
                        || !match relation {
                            CastRelation::You => source.controller == player,
                            CastRelation::Opponent => source.controller != player,
                            CastRelation::Any => true,
                        }
                    {
                        continue;
                    }
                    let cycling = cycled.is_some_and(|reference| {
                        self.state.objects[reference.card as usize].owner == player
                    });
                    if ability.program.iter().any(|instruction| {
                        matches!(
                            instruction,
                            Instruction::ExileDiscardedWithStash
                                | Instruction::MayExileDiscardedPlay { .. }
                        )
                    }) {
                        for discarded in events
                            .iter()
                            .filter(|event| event.moved && event.before.owner == player)
                        {
                            let prior = self.state.pending_triggers.len();
                            self.queue_discard_trigger(
                                &source,
                                index,
                                DiscardTriggerView {
                                    player,
                                    amount: 1,
                                    cycling,
                                },
                            );
                            if self.state.pending_triggers.len() > prior {
                                self.state
                                    .pending_triggers
                                    .last_mut()
                                    .unwrap()
                                    .reflexive_subject = Some(Box::new(ReflexiveSubjectView {
                                    object: discarded.after,
                                    additional_objects: Vec::new(),
                                }));
                            }
                        }
                        continue;
                    }
                    for _ in 0..if one_or_more { 1 } else { count } {
                        self.queue_discard_trigger(
                            &source,
                            index,
                            DiscardTriggerView {
                                player,
                                amount: if one_or_more { count } else { 1 },
                                cycling,
                            },
                        );
                    }
                }
            }
        }
        self.finish_discard_after(after, counts, events);
    }
    fn finish_discard_after(
        &mut self,
        after: Option<DiscardAfter>,
        counts: [u32; 2],
        events: &[ZoneChange],
    ) {
        match after {
            Some(DiscardAfter::NonlandReflexive {
                controller,
                ability,
            }) => {
                let count = events
                    .iter()
                    .filter(|event| {
                        event.moved
                            && event.before.owner == controller
                            && event.before.kinds & 2 == 0
                    })
                    .count();
                if count > 0 {
                    let item = self.state.resolving.as_ref().unwrap().item.clone();
                    let before = self.state.pending_triggers.len();
                    self.queue_reflexive(&item, ability, None);
                    if self.state.pending_triggers.len() > before {
                        let trigger = self.state.pending_triggers.last_mut().unwrap();
                        trigger.reflexive_target_limit =
                            Some(u32::try_from(count).expect("discard count exceeds u32"));
                    }
                }
            }
            Some(DiscardAfter::DrawUnlessOpponentDiscardedTwoNonland(controller)) => {
                let nonland = events
                    .iter()
                    .filter(|event| {
                        event.moved
                            && event.before.owner == controller.other()
                            && event.before.kinds & 2 == 0
                    })
                    .count();
                if nonland < 2 {
                    self.draw_many(controller, 1);
                }
            }
            Some(DiscardAfter::RememberActual) => {
                self.state.resolving.as_mut().unwrap().moved_count =
                    Some(counts.into_iter().map(u64::from).sum());
            }
            Some(DiscardAfter::Recruit(player, token)) => {
                // Discard replacement destinations do not change the frozen card type.
                if events.iter().any(|event| {
                    event.moved && event.before.owner == player && event.before.kinds & 2 == 0
                }) {
                    self.create_tokens(player, token, 1, false);
                }
            }
            Some(DiscardAfter::ReturnLandFromGraveyard { player, tapped }) => {
                // The printed restriction is FROM YOUR GRAVEYARD, not merely
                // a discarded card tracked into any public replacement zone.
                let moves = events
                    .iter()
                    .filter(|event| {
                        event.moved
                            && event.before.owner == player
                            && event.before.kinds & 2 != 0
                            && event.destination == Zone::Graveyard
                            && self.object_ref(event.after.card) == event.after
                            && self.state.objects[event.after.card as usize].zone == Zone::Graveyard
                            && event.was_card
                    })
                    .map(|event| {
                        let mut movement =
                            self.entry_move(event.after.card, Zone::Battlefield, tapped);
                        movement.controller = player;
                        movement
                    })
                    .collect::<Vec<_>>();
                if !moves.is_empty() {
                    self.stage_entries(moves, None);
                }
            }
            Some(DiscardAfter::ChooseLand { player, tapped }) => {
                // CR400.7j: follow only this effect's first move to a public zone.
                // Hidden replacement destinations cannot be recovered by identity.
                let objects = events
                    .iter()
                    .filter(|event| {
                        event.moved
                            && matches!(
                                event.destination,
                                Zone::Graveyard
                                    | Zone::Exile
                                    | Zone::Battlefield
                                    | Zone::Stack
                                    | Zone::Command
                            )
                    })
                    .map(|event| event.after)
                    .filter(|reference| {
                        self.object_ref(reference.card) == *reference
                            && self.current_kinds(reference.card) & 2 != 0
                            && !self.state.objects[reference.card as usize].is_token()
                    })
                    .collect();
                self.start_discarded_land_choice(player, tapped, objects);
            }
            Some(DiscardAfter::DrawActual { players, bonus }) => {
                for player in [self.state.active, self.state.active.other()] {
                    if players[player.index()] {
                        self.draw_many(
                            player,
                            u128::from(counts[player.index()]) + u128::from(bonus),
                        );
                    }
                }
            }
            Some(DiscardAfter::LoseLife(player, amount)) => self.lose_life(player, amount),
            Some(DiscardAfter::ConniveCounters(object, amount, placed_by))
                if self.on_battlefield(object.card) && self.object_ref(object.card) == object =>
            {
                self.effect_counter_by(
                    object.card,
                    CounterKind::PlusOne,
                    amount,
                    placed_by,
                    replacements::ResumeMode::Program,
                );
            }
            _ => {}
        }
    }
    fn queue_discard_trigger(
        &mut self,
        source: &ObjectView,
        index: usize,
        event: DiscardTriggerView,
    ) {
        if !self.queue_trigger(source, index) {
            return;
        }
        self.state
            .pending_triggers
            .last_mut()
            .unwrap()
            .discard_event = Some(event);
    }
}
