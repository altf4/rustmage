//! Liliana's entry trigger and paid protective activation.
use super::*;

#[test]
fn liliana_other_entries_and_paid_hexproof() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Liliana the Faultless", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Hand);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let discard = put(&mut g, p, "Island", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Island", Zone::Hand);
        let life = g.state.players[p.index()].life;
        g.move_object(own, Zone::Battlefield);
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for invalid in [source, foe] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let foreign_ref = g.object_ref(foreign);
        reject(
            &mut g,
            Action::ChooseAdditionalCost {
                object: foreign_ref,
            },
        );
        contracts::branch_contract(g.clone());
        let discard_ref = g.object_ref(discard);
        big_helpers::checked(
            &mut g,
            Action::ChooseAdditionalCost {
                object: discard_ref,
            },
        );
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        g.finish_cleanup();
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn liliana_planeswalker_entry_and_stale_protection_target() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Liliana the Faultless", Zone::Battlefield);
            let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Hand);
            let discard = put(&mut g, p, "Island", Zone::Hand);
            let life = g.state.players[p.index()].life;
            g.move_object(walker, Zone::Battlefield);
            g.prepare_triggers();
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            big_helpers::checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(walker),
                },
            );
            let discard_ref = g.object_ref(discard);
            big_helpers::checked(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: discard_ref,
                },
            );
            pay(&mut g);
            if stale {
                g.move_object(walker, Zone::Hand);
                g.move_object(walker, Zone::Battlefield);
            }
            big_helpers::settle(&mut g);
            assert_eq!(g.keywords(walker).contains(Keywords::HEXPROOF), !stale);
        }
    }
}

#[test]
fn liliana_dual_type_entry_triggers_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Liliana the Faultless", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let mut recipe = CopyRecipe::new(Card::from_name("Ajani, Caller of the Pride").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(1 | 64),
            ..CopyOverrides::NONE
        });
        g.stage_copy_creation(p, recipe, 1);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}
