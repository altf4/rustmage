//! Focused printed outcomes; shared engine invariants live in core tests.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn suppression_batch_torch_bargain_scry_and_actual_damage_death_replacement() {
    {
        let p = Player::P0;
        for bargain in [false, true] {
            for shield in [false, true] {
                let mut g = scenario(p);
                let target = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
                if shield {
                    g.add_counter(target, CounterKind::Shield, 1);
                }
                let relic = put(&mut g, p, "Cryogen Relic", Zone::Battlefield);
                let torch = put(&mut g, p, "Torch the Tower", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: torch });
                let sacrifice = bargain.then(|| g.object_ref(relic));
                checked(&mut g, Action::ChooseBargain { sacrifice });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                if bargain {
                    for _ in 0..30 {
                        if g.observe(p).library_choice.is_some() {
                            break;
                        }
                        let action = if g.state.pending == Pending::Priority {
                            Action::Pass
                        } else {
                            g.legal_actions()
                                .into_iter()
                                .find(|a| !matches!(a, Action::Concede))
                                .unwrap()
                        };
                        checked(&mut g, action);
                    }
                    let choice = g.observe(p).library_choice.expect("bargain supplies scry");
                    assert!(g.observe(p.other()).library_choice.is_none());
                    checked(
                        &mut g,
                        Action::ArrangeLibrary {
                            card: choice.remaining[0].id,
                            destination: LibraryDestination::Top,
                        },
                    );
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).damage,
                    if shield {
                        0
                    } else if bargain {
                        3
                    } else {
                        2
                    }
                );
                g.move_object(target, Zone::Graveyard);
                assert_eq!(
                    g.object_view(target).zone,
                    if shield { Zone::Graveyard } else { Zone::Exile }
                );
            }
        }
    }
}

#[test]
fn suppression_batch_webstrike_cycling_captures_x_for_exact_optional_target_and_draws() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let jewel = put(&mut g, p.other(), "The Enigma Jewel", Zone::Battlefield);
        let net = put(&mut g, p.other(), "Braided Net", Zone::Battlefield);
        let elite = put(&mut g, p, "Webstrike Elite", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: elite,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        for _ in 0..20 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(jewel),
            }) {
                break;
            }
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, action);
        }
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(net)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(jewel),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(elite).zone, Zone::Graveyard);
        assert_eq!(g.object_view(jewel).zone, Zone::Graveyard);
        assert_eq!(g.object_view(net).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let second = put(&mut g, p, "Webstrike Elite", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: second,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
