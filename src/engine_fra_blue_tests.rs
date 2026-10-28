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
