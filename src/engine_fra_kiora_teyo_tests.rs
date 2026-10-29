//! Source-lifetime and dual-counter interactions for Kiora and Teyo.
use super::*;

fn choose_entry_target(g: &mut Game, card: CardId, target: CardId) {
    cast(g, card, None);
    for _ in 0..48 {
        g.prepare_triggers();
        let wanted = Action::ChooseTarget {
            target: Target::Card(target),
        };
        if g.legal_actions().contains(&wanted) {
            act(g, wanted);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("entry trigger did not offer its target");
}

#[test]
fn fra_kiora_entry_trigger_keeps_its_controller_after_source_moves_or_changes_control() {
    for p in [Player::P0, Player::P1] {
        for change_control in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Kiora of Fire and Ashes", Zone::Hand);
            cast(&mut g, source, None);
            for _ in 0..24 {
                if g.state.stack.iter().any(|item| {
                    item.source.is_some_and(|identity| identity.id == source) && matches!(item.kind, StackKind::TriggeredAbility { .. })
                }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(g.state.stack.iter().any(|item| item.source.is_some_and(|identity| identity.id == source)
                && matches!(item.kind, StackKind::TriggeredAbility { .. })));
            super::super::contracts::branch_contract(g.clone());
            if change_control {
                g.change_controller(g.object_ref(source), p.other());
            } else {
                g.move_object(source, Zone::Exile);
            }
            settle(&mut g);
            let dragons: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Battlefield && o.is_token() && o.card == Card::test_index(5288)
                })
                .collect();
            assert_eq!(dragons.len(), 1);
            assert_eq!(dragons[0].1.controller, p);
            assert_eq!(dragons[0].1.owner, p);
            assert_eq!(stats(&g, dragons[0].0 as CardId), (5, 5));
        }
    }
}

#[test]
fn fra_teyo_can_target_itself_and_its_keyword_expires() {
    for p in [Player::P0, Player::P1] {
        for (name, keyword) in [
            ("Teyo, Diamondblade Mage", Keywords::DEATHTOUCH),
            ("Teyo, Lightshield Expert", Keywords::HEXPROOF),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            choose_entry_target(&mut g, source, source);
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            assert!(g.keywords(source).contains(keyword));
            g.finish_cleanup();
            assert!(!g.keywords(source).contains(keyword));
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn fra_teyo_dual_type_gets_both_counters_with_doubling_replacement() {
    for p in [Player::P0, Player::P1] {
        for (name, keyword) in [
            ("Teyo, Diamondblade Mage", Keywords::DEATHTOUCH),
            ("Teyo, Lightshield Expert", Keywords::HEXPROOF),
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
            g.add_counter(target, CounterKind::Loyalty, 5);
            g.change_object_form(
                g.object_ref(target),
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((3, 3)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            assert_eq!(g.current_kinds(target) & 65, 65);
            put(&mut g, p, "Doubling Season", Zone::Battlefield);
            let source = put(&mut g, p, name, Zone::Hand);
            choose_entry_target(&mut g, source, target);
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(target, CounterKind::Loyalty), 7);
            assert!(g.keywords(target).contains(keyword));
        }
    }
}
