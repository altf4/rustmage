use super::*;

#[test]
fn gallia_graveyard_cost_entry_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for double in [false, true] {
            for stale in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, "Gallia, Tragic Host", Zone::Graveyard);
                let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
                let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let land = put(&mut g, p, "Forest", Zone::Graveyard);
                if double {
                    put(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                for id in [source, foreign, land] {
                    let object = g.object_ref(id);
                    reject(&mut g, Action::ChooseAdditionalCost { object });
                }
                let object = g.object_ref(own);
                act(&mut g, Action::ChooseAdditionalCost { object });
                pay(&mut g);
                assert_eq!(g.object_view(own).zone, Zone::Exile);
                let saved = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    if stale {
                        g.move_object(source, Zone::Exile);
                        g.move_object(source, Zone::Graveyard);
                    }
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(source).zone,
                        if stale {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                    if !stale {
                        assert!(g.object_view(source).tapped);
                        assert!(g.keywords(source).contains(Keywords::MENACE));
                        assert_eq!(
                            g.counter_count(source, CounterKind::PlusOne),
                            if double { 2 } else { 1 }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn gallia_rejects_activation_without_other_creature_or_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gallia, Tragic Host", Zone::Graveyard);
        let activate = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activate.clone());
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        reject(&mut g, activate);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
    }
}
