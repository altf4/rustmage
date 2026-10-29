use super::*;

fn mind_finish(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }
            | Action::ChooseReplacement { .. }));
    }
    panic!("Way continuation did not finish: {:?}", g.state.pending);
}

fn mind_jace(g: &mut Game, owner: Player) -> CardId {
    g.create_tokens(owner, Card::JacePlaneswalkerToken, 1, false);
    g.state.objects.iter().enumerate().find_map(|(id, o)|
        (o.is_token() && o.zone == Zone::Battlefield && o.controller == owner
            && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap()
}

#[test]
fn mind_sculptor_paid_cast_empowers_five_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let way = put(&mut g, p, "Way of the Mind Sculptor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: way });
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            mind_finish(&mut g);
            let walker = g.state.objects.iter().enumerate().find_map(|(id, o)|
                (o.is_token() && o.zone == Zone::Battlefield && o.controller == p
                    && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap();
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 5);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn mind_sculptor_committed_threshold_survives_countering_and_source_departure() {
    for actor in [Player::P0, Player::P1] {
        for (ability, mine, expected) in [(0, true, 0), (1, true, 1), (1, false, 0)] {
            let mut g = scenario(actor);
            let way = put(&mut g, if mine { actor } else { actor.other() }, "Way of the Mind Sculptor", Zone::Battlefield);
            let walker = mind_jace(&mut g, actor);
            g.state.objects[walker as usize].counters.loyalty = 5;
            act(&mut g, Action::AnnounceActivate { card: walker, ability, counter_payment: Counters::default() });
            pay(&mut g);
            g.prepare_triggers();
            let trigger_count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
                .filter(|item| item.card == way && matches!(item.kind, StackKind::TriggeredAbility { .. })).count();
            assert_eq!(trigger_count, expected);
            let id = g.state.stack.iter().find(|item| item.card == walker && matches!(item.kind, StackKind::ActivatedAbility { .. })).unwrap().id;
            assert!(g.counter_stack_item(id, false));
            g.move_object(way, Zone::Graveyard);
            let before = g.state.players[actor.index()].library.len();
            let saved = g.snapshot();
            for _ in 0..2 {
                mind_finish(&mut g);
                assert_eq!(g.state.players[actor.index()].library.len(), before - expected);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn mind_sculptor_failed_loyalty_cost_is_atomic_and_granted_ability_counts() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let grantor = put(&mut g, actor, "Kiora of Salt and Sand", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 7;
        let action = Action::AnnouncePlaneswalkerGrant { source: g.object_ref(walker), grantor: g.object_ref(grantor), ability: 0 };
        let saved = g.snapshot();
        reject(&mut g, action.clone());
        assert_eq!(g.snapshot(), saved);
        assert!(!g.state.pending_triggers.iter().any(|item| item.card == way));
        g.state.objects[walker as usize].counters.loyalty = 9;
        act(&mut g, action);
        pay(&mut g);
        g.prepare_triggers();
        assert!(g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .any(|item| item.card == way && matches!(item.kind, StackKind::TriggeredAbility { .. })));
        let id = g.state.stack.iter().find(|item| item.card == walker && matches!(item.kind, StackKind::ActivatedAbility { .. })).unwrap().id;
        assert!(g.counter_stack_item(id, false));
        let before = g.state.players[actor.index()].library.len();
        mind_finish(&mut g);
        assert_eq!(g.state.players[actor.index()].library.len(), before - 1);
    }
}

#[test]
fn mind_sculptor_counts_actual_additional_loyalty_counter_payment() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 5;
        // Synthetic typed cost models a -1 loyalty activation which also removes
        // one loyalty counter as an additional source cost.
        let mut definition = *Card::JacePlaneswalkerToken.activation(0);
        definition.cost.remove = Some((Some(CounterKind::Loyalty), 1));
        let mut item = g.activation_item(actor, walker, 0);
        g.pay_activation_definition(&mut item, [0; 6], Counters::of(CounterKind::Loyalty, 1), 0, &definition);
        assert_eq!(item.loyalty_counters_paid, 2);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        g.committed_activation_observers(&item);
        assert!(g.state.pending_triggers.iter().any(|packet| packet.card == way));
    }
}
