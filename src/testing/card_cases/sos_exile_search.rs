//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid exact exile permissions and revealed search-destination transactions.
use super::big_helpers::{checked, next_turn, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected choice");
}
#[test]
fn sos_effects_scrollsmith_hybrid_targets_normal_paid_cast_and_deadline() {
    {
        let p = Player::P0;
        for white in [false, true] {
            let mut g = scenario(p);
            let shock = put(&mut g, p, "Shock", Zone::Graveyard);
            let land = put(&mut g, p, "Island", Zone::Graveyard);
            let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Shock", Zone::Graveyard);
            g.state.players[p.index()].mana = if white {
                [2, 0, 0, 1, 0, 0]
            } else {
                [1, 0, 0, 2, 0, 0]
            };
            let source = put(&mut g, p, "Practiced Scrollsmith", Zone::Hand);
            cast(&mut g, source, None);
            pause(&mut g);
            for bad in [land, creature, foreign] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bad),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(shock),
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(shock).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, shock));

            g.move_object(source, Zone::Graveyard);
            assert!(g.can_play_exiled(p, shock));
            if white {
                next_turn(&mut g);
                assert!(g.can_play_exiled(p, shock));
                next_turn(&mut g);
                assert!(g.can_play_exiled(p, shock));
                next_turn(&mut g);
                assert!(!g.can_play_exiled(p, shock));
            } else {
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.state.players[p.index()].mana[3] = 1;
                checked(&mut g, Action::AnnounceExile { card: shock });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                finish(&mut g);
                assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
                assert!(!g.can_play_exiled(p, shock));
            }
        }
    }
}
