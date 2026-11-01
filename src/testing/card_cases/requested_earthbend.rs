//! Focused printed outcomes; shared engine invariants live in core tests.
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
fn earthbend_does_not_follow_a_land_that_left_for_hand() {
    {
        let p = Player::P0;
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
fn earthbend_has_no_effect_if_land_changes_controller_before_resolution() {
    {
        let p = Player::P0;
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
fn ascension_entry_earthbends_then_searches_and_sees_the_landfall() {
    {
        let p = Player::P0;
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
