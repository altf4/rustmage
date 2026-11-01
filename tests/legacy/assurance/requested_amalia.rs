//! Ward and life-gain explore use the source's live or last-known characteristics.
use super::big_helpers::{checked, settle, setup};
use super::*;

#[test]
fn amalia_explore_checks_exact_power_after_counter_and_respects_indestructible() {
    for p in [Player::P0, Player::P1] {
        for (growth, destroy) in [(16, false), (17, true), (18, false)] {
            let mut g = setup(p);
            let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let protected = put(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            g.state.objects[source as usize].growth.power = growth;
            g.gain_life(p, 1);
            settle(&mut g);
            assert_eq!(g.stats(source).power, 3 + growth);
            assert_eq!(g.object_view(victim).zone == Zone::Graveyard, destroy);
            assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
            assert_eq!(g.object_view(top).zone, Zone::Library);
        }
    }
}

#[test]
fn amalia_explores_with_current_or_last_controller_and_uses_departure_power() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Island", Zone::Library);
            g.gain_life(p, 1);
            g.prepare_triggers();
            g.state.objects[source as usize].controller = p.other();
            if leave {
                g.state.objects[source as usize].growth.power = 18;
                g.move_object(source, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.object_view(land).zone, Zone::Hand);
            assert_eq!(g.object_view(victim).zone == Zone::Graveyard, leave);
        }
    }
}

#[test]
fn amalia_ward_can_be_paid_or_counter_the_opposing_spell() {
    for p in [Player::P0, Player::P1] {
        for pay_life in [false, true] {
            let mut g = setup(p.other());
            let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            super::big_helpers::cast(&mut g, shock, None, &[Target::Card(source)]);
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::ResolutionLife { .. }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::ResolutionLife { .. }));
            let life = g.state.players[p.other().index()].life;
            checked(
                &mut g,
                if pay_life {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if pay_life { 3 } else { 0 }
            );
            assert_eq!(
                g.object_view(source).zone,
                if pay_life {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn amalia_changed_controller_applies_that_players_counter_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
        put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        put(&mut g, p.other(), "Bear Cub", Zone::Library);
        g.gain_life(p, 1);
        g.prepare_triggers();
        g.state.objects[source as usize].controller = p.other();
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    }
}

#[test]
fn amalia_destruction_consumes_shields_and_honors_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
        let shielded = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Library);
        g.add_counter(shielded, CounterKind::Shield, 1);
        g.state.objects[source as usize].growth.power = 17;
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(g.object_view(shielded).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(shielded, CounterKind::Shield), 0);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn amalia_empty_library_still_explores_and_does_not_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].library.clear();
        g.state.objects[source as usize].growth.power = 17;
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn amalia_two_life_events_create_two_explores_and_checked_choice_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let top = put(&mut g, p, "Bear Cub", Zone::Library);
        g.state.objects[source as usize].growth.power = 17;
        g.gain_life(p, 1);
        g.gain_life(p, 1);
        g.prepare_triggers();
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Explore { .. }) {
                break;
            }
            if matches!(g.state.pending, Pending::Priority) {
                checked(&mut g, Action::Pass);
            } else {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap();
                checked(&mut g, action);
            }
        }
        assert!(matches!(g.state.pending, Pending::Explore { .. }));
        let object = g.object_ref(top);
        checked(
            &mut g,
            Action::ChooseExplore {
                object,
                graveyard: false,
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        assert_eq!(g.stats(source).power, 21);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(top).zone, Zone::Library);
    }
}
