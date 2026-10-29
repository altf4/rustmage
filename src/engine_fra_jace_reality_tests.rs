use super::*;

fn jace_fixture(g: &mut Game, player: Player) -> CardId {
    // The catalogue remains gated; install the authored card in an inert fixture.
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Jace, Reality Sculptor").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}
fn activate_jace(g: &mut Game, source: CardId, ability: u16) {
    act(g, Action::AnnounceActivate { card: source, ability, counter_payment: Counters::default() });
    pay(g);
}

#[test]
fn jace_empowers_using_islands_at_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        put(&mut g, player, "Island", Zone::Battlefield);
        put(&mut g, player.other(), "Island", Zone::Battlefield);
        activate_jace(&mut g, jace, 0);
        put(&mut g, player, "Island", Zone::Battlefield);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 8);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn jace_exiles_everything_above_bottom_even_if_threshold_changes_after_activation() {
    for player in [Player::P0, Player::P1] {
        for library_size in [0, 1, 3] {
            let mut g = scenario(player);
            let jace = jace_fixture(&mut g, player);
            let opponent = player.other();
            let existing = g.state.players[opponent.index()].library.clone();
            for id in existing { g.move_object(id, Zone::Exile); }
            let mut library = Vec::new();
            for _ in 0..library_size { library.push(put(&mut g, opponent, "Forest", Zone::Library)); }
            g.state.objects[jace as usize].counters.loyalty = 24;
            let action = Action::AnnounceActivate { card: jace, ability: 2, counter_payment: Counters::default() };
            reject(&mut g, action.clone());
            g.state.objects[jace as usize].counters.loyalty = 25;
            act(&mut g, action);
            pay(&mut g);
            g.state.objects[jace as usize].counters.loyalty = 1;
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.state.players[opponent.index()].library, library.iter().take(1).copied().collect::<Vec<_>>());
                for &id in library.iter().skip(1) { assert_eq!(g.object_view(id).zone, Zone::Exile); }
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn jace_delayed_trigger_is_per_attacker_and_tracks_exact_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        let walker = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let first = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        activate_jace(&mut g, jace, 1);
        settle(&mut g);
        g.move_object(jace, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: first });
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: second, defender });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        let count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .filter(|item| item.card == jace && matches!(item.kind, StackKind::TriggeredAbility { .. })).count();
        assert_eq!(count, 2);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(second, Zone::Exile);
            g.move_object(second, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(stats(&g, first), (-3, 2));
            assert_eq!(stats(&g, second), (3, 3));
            g.finish_cleanup();
            assert_eq!(stats(&g, first), (2, 2));
            g.restore(&saved).unwrap();
        }
        g.state.active = player;
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
    }
}
