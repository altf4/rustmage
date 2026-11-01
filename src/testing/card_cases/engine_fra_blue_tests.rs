//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn icy_reception_modes_targets_payment_and_cleanup() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let icy = put(&mut g, p, "Icy Reception", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: icy });
        act(&mut g, Action::ChooseMode { mode: 1 });

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
fn icy_uncounterable_and_constrained_opponent_payment() {
    {
        let p = Player::P0;
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
