//! Doran uses the spell's real characteristics and an exact, non-targeted combat subject.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
#[test]
fn ecl_doran_paid_body_reduces_only_qualifying_creature_generic_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doran = named(&mut g, p, "Doran, Besieged by Time", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 1, 0, 1, 1];
        cast(&mut g, doran, None, &[]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [1, 0, 1, 0, 1, 1]
        );
        settle(&mut g);
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 0,
                toughness: 5
            }
        );
        let barrier = named(&mut g, p, "Gleaming Barrier", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        cast(&mut g, barrier, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.on_battlefield(barrier));
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: bear })
        );
        g.state.players[p.index()].mana[5] = 1;
        cast(&mut g, bear, None, &[]);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 2);
        settle(&mut g);
    }
}
#[test]
fn ecl_doran_reducer_requires_current_control_and_available_source_ability() {
    for p in [Player::P0, Player::P1] {
        for blank in [true, false] {
            let mut g = setup(p);
            let doran = named(
                &mut g,
                if blank { p } else { p.other() },
                "Doran, Besieged by Time",
                Zone::Battlefield,
            );
            if blank {
                let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
                cast(&mut g, stop, None, &[Target::Card(doran)]);
                settle(&mut g);
            }
            let barrier = named(&mut g, p, "Gleaming Barrier", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceCast { card: barrier })
            );
            g.state.players[p.index()].mana[5] = 2;
            cast(&mut g, barrier, None, &[]);
            settle(&mut g);
            assert!(g.on_battlefield(barrier));
        }
    }
}
#[test]
fn ecl_doran_each_own_attacker_uses_absolute_difference_without_targeting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doran = named(&mut g, p, "Doran, Besieged by Time", Zone::Battlefield);
        let lion = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[doran, lion, bear]);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
        );
        settle(&mut g);
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 5,
                toughness: 10
            }
        );
        assert_eq!(
            g.stats(lion),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.finish_cleanup();
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 0,
                toughness: 5
            }
        );
        assert_eq!(
            g.stats(lion),
            Stats {
                power: 2,
                toughness: 1
            }
        );
    }
}
#[test]
fn ecl_doran_block_events_belong_to_defending_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let doran = named(&mut g, p, "Doran, Besieged by Time", Zone::Battlefield);
        let barrier = named(&mut g, p, "Gleaming Barrier", Zone::Battlefield);
        let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        attack(&mut g, &[a, b]);
        settle(&mut g);
        assert_eq!(g.stats(doran).power, 0);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.pending, Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: doran,
                attacker: a,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: barrier,
                attacker: b,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        settle(&mut g);
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 5,
                toughness: 10
            }
        );
        assert_eq!(
            g.stats(barrier),
            Stats {
                power: 4,
                toughness: 8
            }
        );
    }
}
#[test]
fn ecl_doran_resolution_uses_current_difference_after_nameless_inversion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doran = named(&mut g, p, "Doran, Besieged by Time", Zone::Battlefield);
        attack(&mut g, &[doran]);
        let inversion = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        cast(&mut g, inversion, None, &[Target::Card(doran)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        settle(&mut g);
        assert_eq!(
            g.stats(doran),
            Stats {
                power: 4,
                toughness: 3
            }
        );
    }
}
#[test]
fn ecl_doran_subject_reentry_source_departure_and_negative_power_are_exact() {
    for p in [Player::P0, Player::P1] {
        for reenter in [true, false] {
            let mut g = setup(p);
            let doran = named(&mut g, p, "Doran, Besieged by Time", Zone::Battlefield);
            let giant = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[giant as usize].growth.power = -5;
            attack(&mut g, &[giant]);
            if reenter {
                let blink = named(&mut g, p, "Personify", Zone::Hand);
                cast(&mut g, blink, None, &[Target::Card(giant)]);
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
            } else {
                let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
                cast(&mut g, bounce, None, &[Target::Card(doran)]);
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
            }
            settle(&mut g);
            assert_eq!(
                g.stats(giant),
                if reenter {
                    Stats {
                        power: 3,
                        toughness: 3,
                    }
                } else {
                    Stats {
                        power: 3,
                        toughness: 8,
                    }
                }
            );
        }
    }
}
