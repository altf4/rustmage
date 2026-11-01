//! Focused printed outcomes; shared engine invariants live in core tests.
//! Complete paid modal and death-trigger discard/draw behavior.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected private discard choice");
}
#[test]
fn sos_followon_borrowed_modes_resolution_hand_size_and_actual_replaced_discards() {
    {
        let p = Player::P0;
        for mode in [0, 1] {
            for empty in [false, true] {
                let mut g = scenario(p);
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
                let a = put(&mut g, p.other(), "Forest", Zone::Hand);
                let b = put(&mut g, p.other(), "Island", Zone::Hand);
                let own: Vec<_> = if empty {
                    vec![]
                } else {
                    vec![
                        put(&mut g, p, "Shock", Zone::Hand),
                        put(&mut g, p, "Island", Zone::Hand),
                    ]
                };
                let spell = put(&mut g, p, "Borrowed Knowledge", Zone::Hand);
                g.state.players[p.index()].cards_discarded_this_turn = 27;
                let targets = [Target::Player(p.other())];
                paid(
                    &mut g,
                    spell,
                    Some(mode),
                    if mode == 0 { &targets } else { &[] },
                );
                // Resolve using the current opponent hand count, not cast-time state.
                g.move_object(a, Zone::Graveyard);
                if mode == 0 && !empty {
                    pause(&mut g);
                    assert!(
                        g.observe(p.other())
                            .objects
                            .iter()
                            .all(|o| !own.contains(&o.id))
                    );
                    for &card in &own {
                        checked(&mut g, Action::Discard { card });
                    }
                }
                finish(&mut g);
                let expected = if mode == 0 { 1 } else { own.len() };
                assert_eq!(g.hand_count(p), expected);
                assert_eq!(g.object_view(b).zone, Zone::Hand);
                for id in own {
                    assert_eq!(g.object_view(id).zone, Zone::Exile);
                }
                assert_eq!(g.object_view(spell).zone, Zone::Exile);
            }
        }
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Borrowed Knowledge", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.hand_count(p), 0);
    }
}
