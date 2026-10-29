//! Surrak's target events, controller restrictions and uncounterability.
use super::big_helpers::{cast as paid_cast, checked, settle, setup};
use super::*;

#[test]
fn surrak_draws_when_opponents_target_its_controllers_creatures() {
    for p in [Player::P0, Player::P1] {
        for own_target in [false, true] {
            let mut g = setup(p.other());
            let surrak = put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
            let creature = put(
                &mut g,
                if own_target { p.other() } else { p },
                "Grizzly Bears",
                Zone::Battlefield,
            );
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            paid_cast(&mut g, shock, None, &[Target::Card(creature)]);
            // The event has already happened: removing the source cannot undo it.
            g.move_object(surrak, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(!own_target)
            );
        }
    }
}

#[test]
fn surrak_triggers_for_creature_spells_targeted_on_the_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        paid_cast(&mut g, creature, None, &[]);
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        paid_cast(&mut g, counter, None, &[Target::Spell(creature)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}

#[test]
fn surrak_spell_cannot_be_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let surrak = put(&mut g, p, "Surrak, Elusive Hunter", Zone::Hand);
        paid_cast(&mut g, surrak, None, &[]);
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        paid_cast(&mut g, counter, None, &[Target::Spell(surrak)]);
        settle(&mut g);
        assert_eq!(g.object_view(surrak).zone, Zone::Battlefield);
        assert!(g.keywords(surrak).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn surrak_triggers_for_an_opponents_activated_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let passage = put(&mut g, p.other(), "Rogue's Passage", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: passage,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
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
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn surrak_copied_provider_is_registered_and_removed_with_the_copy() {
    for p in [Player::P0, Player::P1] {
        for copy_surrak in [false, true] {
            let mut g = setup(p.other());
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if copy_surrak {
                g.apply_temporary_copy(
                    g.object_ref(source),
                    CopyRecipe::new(Card::from_name("Surrak, Elusive Hunter").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            } else {
                g.apply_temporary_copy(
                    g.object_ref(source),
                    CopyRecipe::new(Card::from_name("Surrak, Elusive Hunter").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
                g.apply_temporary_copy(
                    g.object_ref(source),
                    CopyRecipe::new(Card::from_name("Grizzly Bears").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            paid_cast(&mut g, shock, None, &[Target::Card(victim)]);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(copy_surrak)
            );
        }
    }
}

#[test]
fn surrak_loses_target_trigger_while_abilities_are_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let surrak = put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
        let wasp = put(&mut g, p, "The Wondrous Wasp", Zone::Hand);
        paid_cast(&mut g, wasp, None, &[]);
        g.prepare_triggers();
        for _ in 0..32 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(surrak),
            }) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(surrak),
                    },
                );
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        assert!(!g.printed_abilities_available(surrak));
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        paid_cast(&mut g, shock, None, &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn surrak_retarget_event_counts_each_new_object_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        paid_cast(&mut g, shock, None, &[Target::Card(a)]);
        let item = g
            .state
            .stack
            .iter()
            .find(|s| {
                s.source.is_some_and(|source| source.id == shock) && s.kind == StackKind::Spell
            })
            .unwrap()
            .clone();
        let before = g.state.pending_triggers.len();
        let mut copy = item.clone();
        let bound_a = g.bind_target(Target::Card(a));
        let bound_b = g.bind_target(Target::Card(b));
        copy.targets = vec![bound_a, bound_a];
        g.acquire_stack_target_set(&copy, &[]);
        assert_eq!(
            g.state.pending_triggers.len(),
            before + 1,
            "same object twice is one acquisition"
        );
        copy.targets = vec![bound_a, bound_b];
        g.acquire_stack_target_set(&copy, &[bound_a]);
        assert_eq!(
            g.state.pending_triggers.len(),
            before + 2,
            "only the newly targeted creature triggers"
        );
        g.acquire_stack_target_set(&copy, &[bound_a, bound_b]);
        assert_eq!(
            g.state.pending_triggers.len(),
            before + 2,
            "unchanged targets do not trigger"
        );
    }
}
