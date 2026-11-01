//! A private borrowed entry projection shares Supper's exact layer-four form.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn supper_borrowed_entry_projection_food_types_and_retained_mana_match_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // Fixture admission is explicit; these are also in the cohort whitelist.
        let elf = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let automaton = named(&mut g, p.other(), "Adaptive Automaton", Zone::Graveyard);
        let granting_card = Card::from_name("Supper for Spiders").expect("named fixture");
        assert!(granting_card.is_supported());
        let mut moves = vec![];
        for id in [elf, automaton] {
            let mut m = g.entry_move(id, Zone::Battlefield, false);
            m.controller = p;
            m.food_return = Some(granting_card);
            moves.push(m);
        }
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        let mut saw = false;
        world.visit_entry_moves(moves, true, None, &mut |next| {
            assert_eq!(CharacteristicWorld::kinds(next, elf), 16);
            assert_eq!(CharacteristicWorld::kinds(next, automaton), 16);
            assert!(
                next.subtypes(elf)
                    .has(Card::from_name("Llanowar Elves").unwrap(), "Food")
            );
            assert!(
                !next
                    .subtypes(elf)
                    .has(Card::from_name("Llanowar Elves").unwrap(), "Elf")
            );
            assert!(
                !next
                    .subtypes(automaton)
                    .has(Card::from_name("Adaptive Automaton").unwrap(), "Artificer")
            );
            let q = characteristic_query::CharacteristicQuery::in_world(next);
            assert_eq!(q.kinds(elf), 16);
            let resource = next.resources.iter().find(|r| r.id == elf).unwrap();
            assert!(resource.object.summoning_sick);
            saw = true;
            true
        });
        assert!(saw);
        assert_eq!(g.snapshot(), saved);
    }
}
