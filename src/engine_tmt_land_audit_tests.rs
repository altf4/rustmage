//! Rules audit of the five inherited TMT gainlands, using real land plays.
use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::tmt_departure_tests::reject;
use super::*;
use crate::cards::AnimationDuration;
const LANDS: [(&str, [usize; 2]); 5] = [
    ("Dimension X", [3, 0]),
    ("Foot Headquarters", [0, 2]),
    ("Illegitimate Business", [2, 4]),
    ("Mutant Town", [4, 1]),
    ("TCRI Building", [1, 3]),
];
fn own_main(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    while g.state.step != Step::Main1 {
        checked(g, Action::Pass);
    }
}
fn activates(g: &Game, card: CardId) -> Vec<Action> {
    g.legal_actions().into_iter().filter(|a|matches!(a,Action::ActivateMana {card:c,..}|Action::AnnounceMana {card:c,..} if *c==card)).collect()
}
#[test]
fn tmt_land_audit_all_five_play_tapped_and_life_is_a_resolving_trigger() {
    for p in [Player::P0, Player::P1] {
        for (name, _) in LANDS {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let land = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].life;
            checked(&mut g, Action::PlayLand { card: land });
            assert!(g.state.objects[land as usize].tapped);
            assert_eq!(g.state.players[p.index()].life, before);
            assert!(activates(&g, land).is_empty());
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, before + 1);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
#[test]
fn tmt_land_audit_exact_colors_share_one_tap_cost_and_do_not_use_stack() {
    for p in [Player::P0, Player::P1] {
        for (name, colors) in LANDS {
            for color in colors {
                let mut g = setup(p);
                let land = named(&mut g, p, name, Zone::Hand);
                checked(&mut g, Action::PlayLand { card: land });
                settle(&mut g);
                own_main(&mut g);
                assert!(!g.state.objects[land as usize].tapped);
                g.state.players[p.index()].mana = [0; 6];
                let options: Vec<_> = activates(&g, land)
                    .into_iter()
                    .filter_map(|a| match a {
                        Action::ActivateMana { production, .. } => Some(production),
                        _ => None,
                    })
                    .collect();
                assert_eq!(options.len(), 2);
                for mana in &options {
                    assert_eq!(mana.iter().sum::<u32>(), 1);
                    assert!(colors.iter().any(|&c| mana[c] == 1));
                }
                let mut production = [0; 6];
                production[color] = 1;
                checked(
                    &mut g,
                    Action::ActivateMana {
                        card: land,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production,
                    },
                );
                assert_eq!(g.state.players[p.index()].mana, production);
                assert!(g.state.stack.is_empty());
                assert!(activates(&g, land).is_empty());
                reject(
                    &mut g,
                    Action::ActivateMana {
                        card: land,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production,
                    },
                );
            }
        }
    }
}
#[test]
fn tmt_land_audit_borrowed_funding_obeys_tap_and_each_color() {
    for p in [Player::P0, Player::P1] {
        for (name, colors) in LANDS {
            let mut g = setup(p);
            let land = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, Action::PlayLand { card: land });
            settle(&mut g);
            let goal = |color| {
                let mut required = [0; 6];
                required[color] = 1;
                FundingGoal {
                    actor: p,
                    required,
                    generic: 0,
                    source: None,
                    reserve: None,
                }
            };
            for color in 0..6 {
                assert!(!g.can_fund(goal(color)));
            }
            own_main(&mut g);
            for color in 0..6 {
                assert_eq!(g.can_fund(goal(color)), colors.contains(&color));
            }
            let r = g.object_ref(land);
            g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
            for color in 0..6 {
                assert!(!g.can_fund(goal(color)));
            }
            assert!(activates(&g, land).is_empty());
        }
    }
}
#[test]
fn tmt_land_audit_entry_trigger_survives_departure_and_reentry_is_tapped_again() {
    for p in [Player::P0, Player::P1] {
        for (name, _) in LANDS {
            let mut g = setup(p);
            let land = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].life;
            checked(&mut g, Action::PlayLand { card: land });
            g.move_object(land, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, before + 1);
            reject(&mut g, Action::PlayLand { card: land });
            own_main(&mut g);
            checked(&mut g, Action::PlayLand { card: land });
            assert!(g.state.objects[land as usize].tapped);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, before + 2);
        }
    }
}
#[test]
fn tmt_land_audit_ability_removal_does_not_erase_already_stacked_entry_trigger() {
    for p in [Player::P0, Player::P1] {
        for (name, _) in LANDS {
            let mut g = setup(p);
            let land = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].life;
            checked(&mut g, Action::PlayLand { card: land });
            let r = g.object_ref(land);
            g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, before + 1);
            own_main(&mut g);
            assert!(activates(&g, land).is_empty());
        }
    }
}
