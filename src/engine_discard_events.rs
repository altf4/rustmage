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
    LoseLife(Player, i128),
    ConniveCounters(ObjectRef, u64),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscardContext {
    sources: Vec<ObjectView>,
    cycled: Option<ObjectRef>,
    shuffles: Vec<ObjectRef>,
    pub after: Option<DiscardAfter>,
}
impl DiscardContext {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.sources.len() * std::mem::size_of::<ObjectView>()
            + self
                .sources
                .iter()
                .map(|s| {
                    s.counters.owned_bytes()
                        + s.abilities.as_ref().map_or(0, |a| {
                            std::mem::size_of::<AbilityAvailabilityView>()
                                + a.grants.len() * std::mem::size_of::<GrantedAbilityView>()
                        })
                })
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
        if cards.is_empty() {
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
            shuffles: Vec::new(),
            after: None,
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
        let events = self.move_objects_raw(cards, Zone::Graveyard);
        self.finish_discard_events(&events, context);
    }
    pub(super) fn finish_discard_events(&mut self, events: &[ZoneChange], context: DiscardContext) {
        let DiscardContext {
            sources,
            cycled,
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
                    self.queue_discard_trigger(
                        &source,
                        index,
                        DiscardTriggerView {
                            player,
                            amount: 1,
                            cycling,
                        },
                    );
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
                    if ability
                        .program
                        .contains(&Instruction::ExileDiscardedWithStash)
                    {
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
        match after {
            Some(DiscardAfter::LoseLife(player, amount)) => self.lose_life(player, amount),
            Some(DiscardAfter::ConniveCounters(object, amount))
                if self.on_battlefield(object.card) && self.object_ref(object.card) == object =>
            {
                self.effect_counter(
                    object.card,
                    CounterKind::PlusOne,
                    amount,
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
