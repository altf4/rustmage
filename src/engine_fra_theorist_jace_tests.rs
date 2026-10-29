use super::*;

#[test]
fn theorist_opponent_draw_step_trigger_tracks_control_and_queued_source() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player.other());
        let source = put(&mut g, player, "The Theorist, Jace Beleren", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 3;
        put(&mut g, player, "Island", Zone::Library);
        let before = g.state.players[player.index()].cards_drawn_this_turn;
        g.begin_step(Step::Draw);
        g.prepare_triggers();
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, before + 1);
            assert_eq!(g.state.players[player.other().index()].cards_drawn_this_turn, 0);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn theorist_plus_one_makes_blue_illusion_and_replays() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "The Theorist, Jace Beleren", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 3;
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            let token = g.state.objects.iter().enumerate().find_map(|(id, object)|
                (object.zone == Zone::Battlefield && object.card == Card::test_index(5464)).then_some(id as CardId)).unwrap();
            assert_eq!(g.object_view(token).controller, player);
            assert_eq!(stats(&g, token), (1, 1));
            assert!(g.has_subtype(token, "Illusion"));
            assert_eq!(g.counter_count(source, CounterKind::Loyalty), 4);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn theorist_minus_two_accepts_zero_or_opponents_artifact_and_rechecks_target() {
    for player in [Player::P0, Player::P1] {
        for choice in 0..3 {
            let mut g = scenario(player);
            let source = put(&mut g, player, "The Theorist, Jace Beleren", Zone::Battlefield);
            g.state.objects[source as usize].counters.loyalty = 4;
            let own = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, player.other(), "Goldvein Pick", Zone::Battlefield);
            act(&mut g, Action::AnnounceActivate { card: source, ability: 1, counter_payment: Counters::default() });
            reject(&mut g, Action::ChooseTarget { target: Target::Card(own) });
            if choice != 0 { act(&mut g, Action::ChooseTarget { target: Target::Card(target) }); }
            if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
            pay(&mut g);
            if choice == 2 {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, if choice == 1 { Zone::Hand } else { Zone::Battlefield });
                assert_eq!(g.object_view(own).zone, Zone::Battlefield);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn theorist_minus_six_uses_live_post_draw_hand_for_all_controlled_creatures() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "The Theorist, Jace Beleren", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 7;
        let a = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        for _ in 0..3 { put(&mut g, player, "Island", Zone::Library); }
        act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 3);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 3);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            g.restore(&saved).unwrap();
        }
    }
}
