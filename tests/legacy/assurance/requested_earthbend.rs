//! Earthbend animation, counter placement, and exact delayed returns.
use super::big_helpers::{checked, settle};
use super::*;

fn bend(g: &mut Game, land: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, payment);
    settle(g);
}

#[test]
fn ba_sing_se_earthbends_and_returns_the_exact_land_tapped() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Graveyard, Zone::Exile] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Ba Sing Se", Zone::Battlefield);
            let target = put(&mut g, p, "Forest", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: land,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            settle(&mut g);
            assert!(g.is_creature(target));
            assert_ne!(g.current_kinds(target) & 2, 0);
            assert_eq!(
                g.stats(target),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(g.keywords(target).contains(Keywords::HASTE));
            assert_eq!(g.observe(p).earthbend_returns.len(), 1);
            let saved = g.snapshot();
            let mut branch = g.clone();
            branch.restore(&saved).unwrap();
            assert_eq!(
                branch.observe(p.other()).earthbend_returns,
                g.observe(p.other()).earthbend_returns
            );
            g.move_object(land, Zone::Graveyard);
            g.move_object(target, destination);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert!(g.object_view(target).tapped);
            assert!(!g.is_creature(target));
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
            assert!(g.observe(p).earthbend_returns.is_empty());
        }
    }
}

#[test]
fn earthbend_does_not_follow_a_land_that_left_for_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ba Sing Se", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        bend(&mut g, source, land);
        g.move_object(land, Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        g.move_object(land, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    }
}

#[test]
fn repeated_earthbend_and_counter_doubling_create_independent_return_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ba Sing Se", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        bend(&mut g, source, land);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 4);
        g.state.objects[source as usize].tapped = false;
        bend(&mut g, source, land);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 8);
        assert_eq!(g.observe(p).earthbend_returns.len(), 2);
        g.move_object(land, Zone::Graveyard);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.kind == StackKind::DelayedReturn)
                .count(),
            2
        );
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(g.object_view(land).tapped);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
    }
}

#[test]
fn earthbend_has_no_effect_if_land_changes_controller_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ba Sing Se", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        g.state.objects[land as usize].controller = p.other();
        settle(&mut g);
        assert!(!g.is_creature(land));
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
        assert!(g.observe(p).earthbend_returns.is_empty());
    }
}

#[test]
fn ascension_reflexive_requires_threshold_and_uses_departure_counter_lki() {
    for p in [Player::P0, Player::P1] {
        for (initial, doubled, remove_before_departure, expected) in [
            (2, false, false, false),
            (3, false, false, true),
            (2, true, false, true),
            (3, false, true, false),
        ] {
            let mut g = scenario(p);
            let ascension = put(&mut g, p, "Earthbender Ascension", Zone::Battlefield);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            g.add_counter(ascension, CounterKind::Quest, initial);
            let land = put(&mut g, p, "Forest", Zone::Hand);
            g.move_object(land, Zone::Battlefield);
            g.prepare_triggers();
            if initial == 2 && !doubled {
                settle(&mut g);
                assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
                assert!(!g.keywords(creature).contains(Keywords::TRAMPLE));
                continue;
            }
            for _ in 0..40 {
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(creature),
                }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            if remove_before_departure {
                let mut counters = g.counter_amounts(ascension).to_owned();
                counters.set(CounterKind::Quest, 3);
                g.set_counters(ascension, counters);
            }
            g.move_object(ascension, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(
                g.counter_count(creature, CounterKind::PlusOne) > 0,
                expected
            );
            assert_eq!(g.keywords(creature).contains(Keywords::TRAMPLE), expected);
        }
    }
}

#[test]
fn ascension_entry_earthbends_then_searches_and_sees_the_landfall() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Forest", Zone::Battlefield);
        let ascension = put(&mut g, p, "Earthbender Ascension", Zone::Hand);
        super::big_helpers::cast(&mut g, ascension, None, &[]);
        for _ in 0..40 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        for _ in 0..40 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSearch { .. }))
            {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let card = g.state.players[p.index()].library[0];
        checked(&mut g, Action::ChooseSearch { card });
        if g.legal_actions().contains(&Action::FinishSearch) {
            checked(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert!(g.is_creature(target));
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(ascension, CounterKind::Quest), 1);
        assert_eq!(g.state.players[p.index()].library.len(), 7);
    }
}

#[test]
fn earthbend_keeps_land_type_when_earlier_land_effect_ends() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ba Sing Se", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(target));
        assert_eq!(g.current_kinds(target) & 3, 2);
        bend(&mut g, source, target);
        assert_eq!(g.current_kinds(target) & 3, 3);
        let before = g.snapshot();
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.current_kinds(target) & 3, 3);
        assert_eq!(
            g.stats(target),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let mut branch = g.clone();
        branch.restore(&before).unwrap();
        branch.move_object(aura, Zone::Graveyard);
        assert_eq!(branch.snapshot(), g.snapshot());
    }
}
