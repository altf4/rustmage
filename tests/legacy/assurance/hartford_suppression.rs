use super::big_helpers::{cast as paid_cast, checked, settle};
use super::*;

#[test]
fn suppression_batch_tidebinder_counters_entry_ability_and_blanks_exact_source_until_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let relic = put(&mut g, p, "Cryogen Relic", Zone::Hand);
        paid_cast(&mut g, relic, None, &[]);
        for _ in 0..20 {
            if g.state.pending == Pending::Priority
                && g.state
                    .stack
                    .last()
                    .is_some_and(|item| item.kind != StackKind::Spell)
            {
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
        let ability = g.state.stack.last().unwrap().id;
        assert_ne!(g.state.stack.last().unwrap().kind, StackKind::Spell);
        // Pass the relic controller's priority to the opposing flash creature.
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        let binder = put(&mut g, p.other(), "Tishana's Tidebinder", Zone::Hand);
        paid_cast(&mut g, binder, None, &[]);
        // Its optional entry target is announced only after the creature resolves.
        for _ in 0..20 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Stack(ability),
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
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(ability),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(!g.printed_abilities_available(relic));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == relic))
        );
        g.move_object(binder, Zone::Graveyard);
        assert!(g.printed_abilities_available(relic));
        // A later incarnation of Tidebinder cannot recreate the expired duration.
        g.move_object(binder, Zone::Battlefield);
        assert!(g.printed_abilities_available(relic));
    }
}

#[test]
fn suppression_batch_black_sun_pays_bb_and_removes_indestructible_and_death_triggers_before_destroying()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::Indestructible, 1);
        let freebooter = put(&mut g, p.other(), "Greedy Freebooter", Zone::Battlefield);
        let survivor = put(&mut g, p, "Sunspine Lynx", Zone::Battlefield);
        let sun = put(&mut g, p, "Day of Black Sun", Zone::Hand);
        let mut no_black = g.clone();
        no_black.state.players[p.index()].mana = [8, 8, 0, 8, 8, 8];
        assert!(
            !no_black
                .legal_actions()
                .contains(&Action::AnnounceCast { card: sun })
        );
        checked(&mut g, Action::AnnounceCast { card: sun });
        checked(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(freebooter).zone, Zone::Graveyard);
        assert_eq!(g.object_view(survivor).zone, Zone::Battlefield);
        assert!(g.printed_abilities_available(survivor));
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.name() == "Treasure")
        );
    }
}
