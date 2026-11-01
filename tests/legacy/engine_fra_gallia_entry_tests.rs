use super::*;

#[test]
fn gallia_return_counter_exists_for_power_based_entry_observer() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Garruk's Uprising", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let gallia = put(&mut g, p, "Gallia, Tragic Host", Zone::Graveyard);
            let cost = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
            let before = g.state.players[p.index()].library.len();
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: gallia,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(cost);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(gallia).zone, Zone::Battlefield);
                assert!(g.object_view(gallia).tapped);
                assert_eq!(
                    g.counter_count(gallia, CounterKind::PlusOne),
                    if doubled { 2 } else { 1 }
                );
                assert_eq!(stats(&g, gallia), if doubled { (4, 3) } else { (3, 2) });
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    before - usize::from(doubled)
                );
            }
        }
    }
}

#[test]
fn gallia_cast_from_hand_enters_without_return_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gallia = put(&mut g, p, "Gallia, Tragic Host", Zone::Hand);
        cast(&mut g, gallia, None);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(gallia).zone, Zone::Battlefield);
            assert!(!g.object_view(gallia).tapped);
            assert_eq!(g.counter_count(gallia, CounterKind::PlusOne), 0);
            assert_eq!(stats(&g, gallia), (2, 1));
            assert!(g.keywords(gallia).contains(Keywords::MENACE));
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: gallia,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
    }
}

#[test]
fn gallia_opponent_turn_activation_and_foreign_copy_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let gallia = put(&mut g, p, "Gallia, Tragic Host", Zone::Graveyard);
        let cost = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: gallia,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(cost);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Exile);
        let mut copy = g.state.stack.last().expect("Gallia activation").clone();
        copy.id = g.next_stack_id();
        copy.controller = p.other();
        copy.is_copy = true;
        g.state.stack.push(copy);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(gallia).zone, Zone::Battlefield);
            assert_eq!(g.object_view(gallia).controller, p.other());
            assert!(g.object_view(gallia).tapped);
            assert_eq!(g.counter_count(gallia, CounterKind::PlusOne), 2);
            assert_eq!(g.object_view(cost).zone, Zone::Exile);
        }
    }
}
