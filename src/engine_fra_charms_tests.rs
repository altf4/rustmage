use super::*;

#[test]
fn konstrari_charm_modes_targets_and_mana() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let flying = put(&mut g, p.other(), "Ajani's Pridemate", Zone::Battlefield);
            let ground = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[flying as usize].gained_keywords = Keywords::FLYING;
            let charm = put(&mut g, p, "Konstrari Charm", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 0];
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ground),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(flying),
                    },
                );
            } else if mode == 1 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ground),
                    },
                );
            }
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            settle(&mut g);
            match mode {
                0 => assert_eq!(g.object_view(flying).zone, Zone::Graveyard),
                1 => {
                    assert_eq!(g.counter_count(ground, CounterKind::PlusOne), 2);
                    assert!(g.keywords(ground).contains(Keywords::TRAMPLE));
                    g.finish_cleanup();
                    assert!(!g.keywords(ground).contains(Keywords::TRAMPLE));
                    assert_eq!(g.counter_count(ground, CounterKind::PlusOne), 2);
                }
                _ => assert_eq!(g.state.players[p.index()].mana[5], 3),
            }
        }
    }
}

#[test]
fn charge_the_sanctum_hybrid_modes_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (mode, color) in [(0, 0), (1, 3)] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let charm = put(&mut g, p, "Charge the Sanctum", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 2;
            act(&mut g, Action::AnnounceCast { card: charm });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 1 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(other),
                    },
                );
            }
            pay(&mut g);
            settle(&mut g);
            if mode == 0 {
                assert_eq!(stats(&g, own), (4, 2));
                assert_eq!(stats(&g, other), (2, 2));
            } else {
                assert_eq!(stats(&g, other), (5, 3));
                assert!(g.keywords(other).contains(Keywords::FIRST_STRIKE));
            }
            g.finish_cleanup();
            assert_eq!(stats(&g, own), (2, 2));
            assert!(!g.keywords(other).contains(Keywords::FIRST_STRIKE));
            assert_eq!(stats(&g, other), if mode == 0 { (2, 2) } else { (3, 3) });
        }
    }
}

#[test]
fn konstrari_flying_target_loses_ability_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bird = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[bird as usize].gained_keywords = Keywords::FLYING;
        let charm = put(&mut g, p, "Konstrari Charm", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: charm });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bird),
            },
        );
        pay(&mut g);
        g.state.objects[bird as usize].gained_keywords = Keywords::NONE;
        settle(&mut g);
        assert_eq!(g.object_view(bird).zone, Zone::Battlefield);
        assert_eq!(g.state.objects[bird as usize].damage, 0);
    }
}

#[test]
fn kiora_dragons_entry_activation_and_replacement() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let kiora = put(&mut g, p, "Kiora of Fire and Ashes", Zone::Hand);
            cast(&mut g, kiora, None);
            settle(&mut g);
            let dragons = |g: &Game| -> Vec<CardId> {
                g.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card == Card(5288)
                    })
                    .map(|(id, _)| id as CardId)
                    .collect()
            };
            let amount = if doubled { 2 } else { 1 };
            assert_eq!(dragons(&g).len(), amount);
            for dragon in dragons(&g) {
                assert_eq!(stats(&g, dragon), (5, 5));
                assert!(g.has_subtype(dragon, "Dragon"));
                assert!(g.keywords(dragon).contains(Keywords::FLYING));
                assert_eq!(g.object_view(dragon).card.colors(), 8);
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: kiora,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: kiora,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            super::contracts::branch_contract(g.clone());
            g.move_object(kiora, Zone::Exile);
            settle(&mut g);
            assert_eq!(dragons(&g).len(), amount * 2);
        }
    }
}

#[test]
fn twisted_fates_independent_targets_partial_legality_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for invalid in 0..3 {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let survivor = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let spell = put(&mut g, p, "Twisted Fates", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            super::contracts::branch_contract(g.clone());
            pay(&mut g);
            if invalid == 1 {
                g.move_object(victim, Zone::Exile);
            }
            if invalid == 2 {
                g.state.players[p.other().index()].hexproof_until_end_turn = true;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if invalid == 1 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.counter_count(survivor, CounterKind::PlusOne),
                if invalid == 2 { 0 } else { 2 }
            );
        }
    }
}
