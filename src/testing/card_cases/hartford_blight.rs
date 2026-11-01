//! Focused printed outcomes; shared engine invariants live in core tests.
//! Nonmodal blight and permanent-card graveyard batch interactions.
use super::big_helpers::{checked, settle};
use super::*;

fn choose_cost(g: &mut Game, blight: bool) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| {
            if blight {
                matches!(r.clause, CastCostClause::Blight { .. })
            } else {
                r.clause == CastCostClause::Mana
            }
        })
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause: index });
}

#[test]
fn requiting_hex_paid_or_declined_is_legal_and_resolves_the_printed_effects() {
    {
        let p = Player::P0;
        for paid in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Requiting Hex", Zone::Hand);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let costly = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let subject = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseBlight { pay: paid });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(costly),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if paid {
                checked(&mut g, Action::BeginOrderedCosts);
                choose_cost(&mut g, true);
                let foreign = g.object_ref(victim);
                reject(&mut g, Action::PayBlight { object: foreign });
                let object = g.object_ref(subject);
                checked(&mut g, Action::PayBlight { object });
                choose_cost(&mut g, false);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(
                g.counter_count(subject, CounterKind::MinusOne),
                u64::from(paid)
            );
            assert_eq!(
                g.state.players[p.index()].life,
                life + if paid { 2 } else { 0 }
            );
        }
    }
}

#[test]
fn moonshadow_observes_one_owned_permanent_card_per_batch_from_any_zone() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Moonshadow", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, source), (1, 1));

        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        let a = put(&mut g, p, "Forest", Zone::Hand);
        let b = put(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let instant = put(&mut g, p, "Murder", Zone::Hand);
        let events = g.move_objects_raw(&[a, b, foreign, instant], Zone::Graveyard);
        g.finish_zone_events(&events, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 5);
        for zone in [Zone::Library, Zone::Exile, Zone::Battlefield] {
            let card = put(&mut g, p, "Bear Cub", zone);
            if zone == Zone::Battlefield {
                g.state.objects[card as usize].controller = p.other();
            }
            g.move_object(card, Zone::Graveyard);
            settle(&mut g);
        }
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let spell = put(&mut g, p, "Murder", Zone::Hand);
        let events = g.move_objects_raw(&[other, spell], Zone::Graveyard);
        g.finish_zone_events(&events, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 2);
    }
}

#[test]
fn moonshadow_replacement_suppression_and_trigger_time_condition() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Moonshadow", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        settle(&mut g);
        let rest = put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let exiled = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(exiled, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(exiled).zone, Zone::Exile);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 6);
        g.move_object(rest, Zone::Exile);
        settle(&mut g);
        let a = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(a, Zone::Graveyard);
        assert!(!g.state.pending_triggers.is_empty());
        // The printed "while" condition is checked when the event happens.
        g.state.objects[source as usize].counters.minus_one = 1;
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        let b = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(b, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 0);
        g.state.objects[source as usize].counters.minus_one = 1;
        let reference = g.object_ref(source);
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(reference);
        assert!(!g.printed_abilities_available(source));
        let c = put(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(c, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::MinusOne), 1);
    }
}

#[test]
fn requiting_hex_copies_preserve_blight_and_indestructible_does_not_prevent_life_gain() {
    {
        let p = Player::P0;
        for paid in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Requiting Hex", Zone::Hand);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            let subject = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseBlight { pay: paid });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if paid {
                checked(&mut g, Action::BeginOrderedCosts);
                choose_cost(&mut g, true);
                let object = g.object_ref(subject);
                checked(&mut g, Action::PayBlight { object });
                choose_cost(&mut g, false);
            }
            pay(&mut g);
            let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: sparks });
            checked(&mut g, Action::ChooseMode { mode: 0 });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(spell),
                },
            );
            pay(&mut g);
            for _ in 0..48 {
                if g.legal_actions().contains(&Action::FinishStackTargets) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            checked(&mut g, Action::FinishStackTargets);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(
                g.state.players[p.index()].life,
                life + if paid { 4 } else { 0 }
            );
            assert_eq!(
                g.counter_count(subject, CounterKind::MinusOne),
                u64::from(paid)
            );
        }
    }
}
