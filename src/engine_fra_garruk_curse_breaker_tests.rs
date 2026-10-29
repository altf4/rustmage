use super::*;

fn garruk_curse_activate(g: &mut Game, source: CardId, ability: u16) {
    act(g, Action::AnnounceActivate { card: source, ability, counter_payment: Counters::default() });
    if g.legal_actions().contains(&Action::FinishTargets) { act(g, Action::FinishTargets); }
    pay(g);
    settle(g);
}

#[test]
fn garruk_curse_beast_and_large_entry_draw() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Garruk, Curse Breaker", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 5;
        garruk_curse_activate(&mut g, source, 1);
        let token = g.state.objects.iter().enumerate().find_map(|(id, object)|
            (object.zone == Zone::Battlefield && object.card == Card::test_index(5239)).then_some(id as CardId)).unwrap();
        assert_eq!(stats(&g, token), (4, 4));
        assert!(g.keywords(token).contains(Keywords::TRAMPLE));
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn garruk_curse_untaps_two_lands_and_rechecks_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Garruk, Curse Breaker", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 5;
        let a = put(&mut g, player, "Forest", Zone::Battlefield);
        let b = put(&mut g, player.other(), "Island", Zone::Battlefield);
        g.state.objects[a as usize].tapped = true;
        g.state.objects[b as usize].tapped = true;
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(a) });
        act(&mut g, Action::ChooseTarget { target: Target::Card(b) });
        if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
        pay(&mut g);
        g.move_object(b, Zone::Exile);
        g.move_object(b, Zone::Battlefield);
        g.state.objects[b as usize].tapped = true;
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.object_view(a).tapped);
            assert!(g.object_view(b).tapped);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn garruk_curse_group_attack_survives_source_and_preserves_exact_attackers() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Garruk, Curse Breaker", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 5;
        let a = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let other = put(&mut g, player, "Healer's Hawk", Zone::Battlefield);
        let walker = put(&mut g, player.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        garruk_curse_activate(&mut g, source, 2);
        assert_eq!(g.observe(player.other()).delayed_opponent_attack_until_next_turn.len(), 1);
        g.move_object(source, Zone::Graveyard);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: a });
        act(&mut g, Action::Attack { card: b });
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: other, defender });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        let count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .filter(|item| item.card == source && matches!(item.kind, StackKind::TriggeredAbility { .. })).count();
        assert_eq!(count, 1);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(b, Zone::Exile);
            g.move_object(b, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(stats(&g, a), (4, 4));
            assert!(g.keywords(a).contains(Keywords::TRAMPLE));
            assert_eq!(stats(&g, b), (3, 3));
            assert_eq!(stats(&g, other), (1, 1));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn garruk_curse_delayed_effect_expires_at_controllers_next_turn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Garruk, Curse Breaker", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 5;
        garruk_curse_activate(&mut g, source, 2);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.active = player.other();
            g.begin_turn();
            assert_eq!(g.state.delayed_opponent_attack_until_next_turn.as_ref().unwrap().len(), 1);
            g.state.active = player;
            g.begin_turn();
            assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
            g.restore(&saved).unwrap();
        }
    }
}
