use super::*;

#[test]
fn icy_reception_modes_targets_payment_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let icy = put(&mut g, p, "Icy Reception", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: icy });
        act(&mut g, Action::ChooseMode { mode: 1 });
        super::contracts::branch_contract(g.clone());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, bear), (-3, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));

        for (name, legal, paid) in [
            ("Bear Cub", true, false),
            ("Bear Cub", true, true),
            ("Ajani, Caller of the Pride", true, false),
            ("Divination", false, false),
        ] {
            let mut g = scenario(p.other());
            let target = put(&mut g, p.other(), name, Zone::Hand);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let icy = put(&mut g, p, "Icy Reception", Zone::Hand);
            cast(&mut g, target, None);
            act(&mut g, Action::Pass);
            act(&mut g, Action::AnnounceCast { card: icy });
            if !legal {
                reject(&mut g, Action::ChooseMode { mode: 0 });
                continue;
            }
            act(&mut g, Action::ChooseMode { mode: 0 });
            let action = Action::ChooseTarget {
                target: Target::Spell(target),
            };
            act(&mut g, action);
            pay(&mut g);
            for _ in 0..8 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            super::contracts::branch_contract(g.clone());
            if paid {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[target as usize].zone,
                if paid {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

fn icy_counter(g: &mut Game, p: Player, target: Target) -> CardId {
    g.state.priority = p;
    let icy = put(g, p, "Icy Reception", Zone::Hand);
    act(g, Action::AnnounceCast { card: icy });
    act(g, Action::ChooseMode { mode: 0 });
    act(g, Action::ChooseTarget { target });
    pay(g);
    icy
}

fn icy_payment(g: &mut Game) {
    for _ in 0..16 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("expected payment choice");
}

#[test]
fn icy_copied_spells_legal_alternatives_and_departure() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Ajani, Caller of the Pride"] {
            let mut g = scenario(p.other());
            let original = put(&mut g, p.other(), name, Zone::Hand);
            cast(&mut g, original, None);
            let template = g
                .state
                .stack
                .iter()
                .find(|s| s.source.is_some_and(|source| source.id == original))
                .unwrap()
                .clone();
            let copy = g.allocate_spell_copy(&template, p.other());
            let id = copy.id;
            g.state.stack.push(copy);
            icy_counter(&mut g, p, Target::Stack(id));
            icy_payment(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            act(&mut g, Action::DeclineResolutionMana);
            settle(&mut g);
            assert!(!g.state.stack.iter().any(|s| s.id == id));
            assert_eq!(g.object_view(original).zone, Zone::Battlefield);
        }
        let mut g = scenario(p.other());
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, bear, None);
        g.state.priority = p.other();
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(creature)));
        g.state.priority = p;
        let icy = put(&mut g, p, "Icy Reception", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: icy });
        act(&mut g, Action::ChooseMode { mode: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(shock),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(bear),
            },
        );
        pay(&mut g);
        let victim = g.state.stack.iter().find(|s| s.source.is_some_and(|source| source.id == bear)).unwrap().id;
        assert!(g.counter_stack_item(victim, false));
        settle(&mut g);
        assert_eq!(g.object_view(icy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn icy_uncounterable_and_constrained_opponent_payment() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, pays) in [
            ("Koma, World-Eater", 0, false),
            ("Bear Cub", 2, false),
            ("Bear Cub", 3, true),
        ] {
            let mut g = scenario(p.other());
            let victim = put(&mut g, p.other(), name, Zone::Hand);
            let private = put(&mut g, p.other(), "Forest", Zone::Hand);
            cast(&mut g, victim, None);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, mana];
            icy_counter(&mut g, p, Target::Spell(victim));
            icy_payment(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            let owner = g.observe(p.other());
            let opponent = g.observe(p);
            assert_eq!(owner.resolution_payment.as_ref().unwrap().cost.generic, 3);
            assert_eq!(
                opponent.resolution_payment.as_ref().unwrap().payer,
                p.other()
            );
            assert!(owner.objects.iter().any(|o| o.id == private));
            assert!(opponent.objects.iter().all(|o| o.id != private));
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 2],
                },
            );
            super::contracts::branch_contract(g.clone());
            if pays {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 3],
                    },
                );
                assert_eq!(g.state.players[p.other().index()].mana[5], 0);
            } else {
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::PayResolutionMana { .. }))
                );
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if pays || name == "Koma, World-Eater" {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
