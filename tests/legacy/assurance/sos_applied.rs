//! Copiable 0/0 Fractal exceptions are distinct from the following six counters.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;
fn fractals(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(i as CardId, "Fractal"))
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn sos_applied_geometry_copies_owned_non_aura_land_artifact_and_cda_with_six_counters() {
    for p in [Player::P0, Player::P1] {
        for name in ["Forest", "Goldvein Pick", "Enigma Drake"] {
            let mut g = scenario(p);
            put(&mut g, p, "Shock", Zone::Graveyard);
            let target = put(&mut g, p, name, Zone::Battlefield);
            g.add_counter(target, CounterKind::PlusOne, 3);
            g.state.objects[target as usize].tapped = true;
            let spell = put(&mut g, p, "Applied Geometry", Zone::Hand);
            paid(&mut g, spell, None, &[Target::Card(target)]);
            finish(&mut g);
            let made = fractals(&g, p);
            assert_eq!(made.len(), 1);
            let copy = made[0];
            assert_eq!(g.object_view(copy).name, name);
            assert_eq!(
                g.current_kinds(copy),
                g.state.objects[target as usize].card.kind_mask() | 1
            );
            assert_eq!(stats(&g, copy), (6, 6));
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
            assert!(!g.object_view(copy).tapped);
            assert!(g.object_view(copy).summoning_sick);
            if name == "Enigma Drake" {
                assert!(g.keywords(copy).contains(Keywords::FLYING));
            }
            let recipe = g.copiable_recipe(g.object_ref(copy));
            assert_eq!(recipe.exceptions.base, Some((0, 0)));
            assert!(recipe.exceptions.fractal);
            // Copying the copy retains its 0/0 exception but not its six counters.
            let next = put(&mut g, p, "Molten Duplication", Zone::Hand);
            paid(&mut g, next, None, &[Target::Card(copy)]);
            finish(&mut g);
            assert_eq!(fractals(&g, p), vec![copy]);
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
        }
    }
}
#[test]
fn sos_applied_geometry_target_legality_and_exact_reincarnation_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let aura = put(&mut g, p, "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(body));
        let spell = put(&mut g, p, "Applied Geometry", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        for card in [aura, foreign] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(body),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(body, Zone::Hand);
        g.move_object(body, Zone::Battlefield);
        finish(&mut g);
        assert!(fractals(&g, p).is_empty());
    }
}
#[test]
fn sos_applied_geometry_replaced_token_batch_waits_for_owned_entry_choices_then_replaces_counters()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Adaptive Automaton", Zone::Hand);
        paid(&mut g, source, None, &[]);
        for _ in 0..12 {
            if g.state.pending != Pending::Priority {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let human =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Human").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value: human });
        finish(&mut g);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = put(&mut g, p, "Applied Geometry", Zone::Hand);
        paid(&mut g, spell, None, &[Target::Card(source)]);
        for _ in 0..12 {
            if g.state.pending != Pending::Priority {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let robot =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Robot").unwrap());
        for _ in 0..2 {
            assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
            reject(&mut g, Action::Pass);
            checked(&mut g, Action::ChooseEntryValue { value: robot });
        }
        finish(&mut g);
        let copies = fractals(&g, p);
        assert_eq!(copies.len(), 2);
        for copy in copies {
            assert_eq!(g.object_view(copy).entry_choice, Some(robot));
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 12);
            assert_eq!(stats(&g, copy), (13, 13));
        }
    }
}
