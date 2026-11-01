//! Repeated printed modes retain ordered targets, exact references, and owned entry continuations.
use super::big_helpers::{checked, settle as finish};
use super::*;
fn paid(g: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    reject(g, Action::ChooseMode { mode: 15 });
    checked(g, Action::ChooseMode { mode });
    targets_and_pay(g, targets);
}
fn targets_and_pay(g: &mut Game, targets: &[Target]) {
    for &target in targets {
        for _ in 0..4 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("unpaid Moment");
}
fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("choice expected");
}
#[test]
fn sos_moment_all_fifteen_multiplicities_are_paid_and_resolve_in_printed_order() {
    for p in [Player::P0, Player::P1] {
        let mut mode = 0;
        for destroy in 0..=4 {
            for reanimate in 0..=4 - destroy {
                let mut g = scenario(p);
                let land = put(&mut g, p.other(), "Island", Zone::Battlefield);
                let instant = put(&mut g, p, "Shock", Zone::Graveyard);
                let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let bodies: Vec<_> = (0..destroy)
                    .map(|_| put(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
                    .collect();
                let returns: Vec<_> = (0..reanimate)
                    .map(|_| put(&mut g, p, "Goldvein Pick", Zone::Graveyard))
                    .collect();
                let spell = put(&mut g, p, "Moment of Reckoning", Zone::Hand);
                assert_eq!(
                    Card::from_name("Moment of Reckoning")
                        .unwrap()
                        .fixed_mana_value(),
                    7
                );
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::ChooseMode { mode });
                if destroy > 0 {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(land),
                        },
                    );
                }
                if destroy == 0 && reanimate > 0 {
                    for bad in [instant, foreign] {
                        reject(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(bad),
                            },
                        );
                    }
                }
                let targets: Vec<_> = bodies
                    .iter()
                    .chain(&returns)
                    .map(|&id| Target::Card(id))
                    .collect();
                targets_and_pay(&mut g, &targets);
                finish(&mut g);
                for body in bodies {
                    assert_eq!(g.object_view(body).zone, Zone::Graveyard);
                }
                for body in returns {
                    assert_eq!(g.object_view(body).zone, Zone::Battlefield);
                }
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                assert_eq!(g.object_view(instant).zone, Zone::Graveyard);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                mode += 1;
            }
        }
        assert_eq!(mode, 15);
    }
}
#[test]
fn sos_moment_repeated_returns_pause_in_chosen_relative_order_without_priority() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        let b = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let spell = put(&mut g, p, "Moment of Reckoning", Zone::Hand);
        paid(&mut g, spell, 2, &[Target::Card(b), Target::Card(a)]);
        pause(&mut g);
        assert_eq!(g.observe(p).entry_value.unwrap().object.card, b);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        reject(&mut g, Action::AnnounceCast { card: shock });
        let robot =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Robot").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value: robot });
        assert_eq!(g.observe(p).entry_value.unwrap().object.card, a);
        reject(&mut g, Action::Pass);
        let human =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Human").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value: human });
        finish(&mut g);
        assert_eq!(g.object_view(b).entry_choice, Some(robot));
        assert_eq!(g.object_view(a).entry_choice, Some(human));
    }
}
#[test]
fn sos_moment_repeated_target_partial_invalidation_and_copy_keep_modes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Moment of Reckoning", Zone::Hand);
        paid(&mut g, spell, 9, &[Target::Card(body), Target::Card(body)]);
        finish(&mut g);
        assert_eq!(g.object_view(body).zone, Zone::Graveyard);
        let mut g = scenario(p);
        put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let returned = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Moment of Reckoning", Zone::Hand);
        paid(
            &mut g,
            spell,
            6,
            &[Target::Card(body), Target::Card(returned)],
        );
        pause(&mut g);
        let object = g.object_ref(artifact);
        checked(&mut g, Action::Sacrifice { object });
        if g.legal_actions().contains(&Action::FinishStackTargets) {
            checked(&mut g, Action::FinishStackTargets);
        }
        assert!(
            g.state
                .stack
                .iter()
                .any(|item| item.is_copy && item.mode == Some(6))
        );
        finish(&mut g);
        assert_eq!(g.object_view(body).zone, Zone::Graveyard);
        assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        let mut g = scenario(p);
        let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let returned = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let spell = put(&mut g, p, "Moment of Reckoning", Zone::Hand);
        paid(
            &mut g,
            spell,
            6,
            &[Target::Card(body), Target::Card(returned)],
        );
        g.move_object(body, Zone::Hand);
        g.move_object(body, Zone::Battlefield);
        finish(&mut g);
        assert_eq!(g.object_view(body).zone, Zone::Battlefield);
        assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
    }
}
