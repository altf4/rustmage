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

#[cfg(test)]
mod balin_result_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};
    #[test]
    fn discard_context_counts_only_owned_elements() {
        let mut context = DiscardContext {
            sources: Vec::new(),
            cycled: None,
            cycling_x: None,
            shuffles: Vec::new(),
            after: None,
        };
        assert_eq!(context.owned_bytes(), 0);
        context.shuffles.push(ObjectRef {
            card: 0,
            generation: 0,
        });
        assert_eq!(context.owned_bytes(), std::mem::size_of::<ObjectRef>());
    }

    #[test]
    fn captured_view_counts_abilities_and_power_up_uses() {
        let mut game = setup(Player::P0);
        let id = named(&mut game, Player::P0, "Forest", Zone::Battlefield);
        let mut view = game.object_view(id);
        view.abilities = None;
        view.power_up_uses.clear();
        let before = view.owned_bytes();
        view.abilities = Some(Box::new(AbilityAvailabilityView {
            printed: true,
            grants: vec![GrantedAbilityView {
                grantor: view.reference,
                granting_card: view.card,
                ability: Some(0),
                static_rule: None,
                grantor_controller: Player::P0,
                timestamp: 0,
            }],
        }));
        view.power_up_uses.push(PowerUpUseView {
            definition: view.card,
            ability: 0,
            borrowed_slot: None,
            origin: None,
            count: 1,
        });
        assert_eq!(
            view.owned_bytes(),
            before
                + std::mem::size_of::<AbilityAvailabilityView>()
                + std::mem::size_of::<GrantedAbilityView>()
                + std::mem::size_of::<PowerUpUseView>()
        );
        let context = DiscardContext {
            sources: vec![view],
            cycled: None,
            cycling_x: None,
            shuffles: Vec::new(),
            after: None,
        };
        assert_eq!(
            context.owned_bytes(),
            std::mem::size_of::<ObjectView>() + context.sources[0].owned_bytes()
        );
    }

    fn root(p: Player) -> Game {
        let mut g = setup(p);
        named(&mut g, p, "Balin, Loremaster", Zone::Battlefield);
        let dwarf = named(&mut g, p, "Dwarven Mauler", Zone::Hand);
        g.move_object(dwarf, Zone::Battlefield);
        g.give_priority();
        g.prepare_triggers();
        for _ in 0..32 {
            if g.legal_actions()
                .contains(&Action::ChooseOptional { accept: true })
            {
                return g;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            g.apply(g.actor().unwrap(), a).unwrap();
        }
        panic!("result fixture did not reach Balin optional");
    }
    #[test]
    fn hob_balin_result_empty_batch_overwrites_previous_result() {
        for p in [Player::P0, Player::P1] {
            let mut g = root(p);
            g.state.resolving.as_mut().unwrap().moved_count = Some(99);
            g.apply(p, Action::ChooseOptional { accept: true }).unwrap();
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
    #[test]
    fn hob_balin_result_linked_entry_pause_counts_completed_not_skipped_moves_and_restores() {
        // Synthetic replacement destination isolates the generic discard-context
        // contract. Balin itself cannot cause the opponent-only Wilt-Leaf entry.
        for p in [Player::P0, Player::P1] {
            let mut g = root(p);
            let adaptive = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            let prevented = named(&mut g, p, "Forest", Zone::Hand);
            let r = g.state.resolving.as_mut().unwrap();
            r.program = &[
                Instruction::DrawMoved,
                Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::MovedCount),
            ];
            r.pc = 0;
            r.moved_count = Some(99);
            g.state.pending = Pending::Priority;
            let mut skip = g.entry_move(prevented, Zone::Graveyard, false);
            skip.skipped = true;
            g.stage_discard_entries(
                vec![
                    g.entry_move(adaptive, Zone::Battlefield, false),
                    g.entry_move(shock, Zone::Graveyard, false),
                    skip,
                ],
                DiscardContext {
                    sources: Vec::new(),
                    cycled: None,
                    cycling_x: None,
                    shuffles: Vec::new(),
                    after: Some(DiscardAfter::RememberActual),
                },
            );
            assert!(g.advance_entries());
            assert!(
                g.state
                    .entry_batch
                    .as_ref()
                    .unwrap()
                    .discard_context
                    .is_some()
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            let saved = g.snapshot();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            drop(g);
            let actions = raw.legal_actions();
            let index = actions
                .iter()
                .position(|a| matches!(a, Action::ChooseEntryValue { .. }))
                .unwrap();
            let action = actions[index].clone();
            let revision = indexed.revision();
            indexed.apply_index(p, revision, index).unwrap();
            raw.apply(p, action.clone()).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.state.players[p.index()].cards_discarded_this_turn, 2);
            assert_eq!(raw.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(raw.state.players[p.other().index()].life, 18);
            assert_eq!(raw.object_view(prevented).zone, Zone::Hand);
            indexed.restore(&saved).unwrap();
            indexed.apply(p, action).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
        }
    }
}

#[cfg(test)]
#[path = "engine_silvan_discard_tests.rs"]
mod silvan_discard_tests;
