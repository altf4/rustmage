//! Focused printed outcomes; shared engine invariants live in core tests.
//! Liliana's entry trigger and paid protective activation.
use super::*;

#[test]
fn liliana_other_entries_and_paid_hexproof() {
    {
        let p = Player::P0;
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
fn liliana_dual_type_entry_triggers_once() {
    {
        let p = Player::P0;
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
