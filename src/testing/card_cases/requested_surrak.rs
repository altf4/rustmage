//! Focused printed outcomes; shared engine invariants live in core tests.
//! Surrak's target events, controller restrictions and uncounterability.
use super::big_helpers::{cast as paid_cast, checked, settle, setup};
use super::*;

#[test]
fn surrak_draws_when_opponents_target_its_controllers_creatures() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn surrak_triggers_for_an_opponents_activated_ability() {
    {
        let p = Player::P0;
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
fn surrak_loses_target_trigger_while_abilities_are_removed() {
    {
        let p = Player::P0;
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
