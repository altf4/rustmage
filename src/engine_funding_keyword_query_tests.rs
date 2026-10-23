//! Borrow-scoped keyword reuse versus uncached and exhaustive layer evaluation.
use super::super::ability_timeline_tests::{named, setup};
use super::*;

fn compare_moves(world: &FundingState<'_>, goal: &FundingGoal) {
    let actual = world.moves(goal);
    let uncached = world.moves_with_keywords(goal, |r| world.keywords(r));
    let exhaustive = ability_evaluation::evaluate_abilities(world);
    let reference =
        world.moves_with_keywords(goal, |r| exhaustive.result.objects[r.id as usize].keywords);
    for expected in [&uncached, &reference] {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected.iter()) {
            assert_eq!(world.action(a), world.action(b));
            let mut left = world.clone();
            let mut right = world.clone();
            left.apply(a);
            right.apply(b);
            assert!(
                left.key(goal) == right.key(goal),
                "complete successor facts differ"
            );
        }
    }
}

#[test]
fn funding_keyword_query_dense_both_seats_matches_uncached_and_exhaustive() {
    for p in [Player::P0, Player::P1] {
        let mut game = setup(p);
        let elves: Vec<_> = (0..24)
            .map(|_| named(&mut game, p, "Llanowar Elves", Zone::Battlefield))
            .collect();
        for &id in &elves {
            game.state.objects[id as usize].summoning_sick = true;
        }
        for _ in 0..12 {
            named(&mut game, p, "Forest", Zone::Battlefield);
        }
        for _ in 0..12 {
            named(&mut game, p, "Skateboard", Zone::Battlefield);
        }
        named(&mut game, p, "Mona Lisa, Science Geek", Zone::Battlefield);
        let spell = named(&mut game, p, "Goatnap", Zone::Graveyard);
        for &id in &elves[..12] {
            game.grant_keyword_effects(&[id], game.object_ref(spell), Keywords::HASTE);
        }
        game.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 1, 0, 0, 0, 0],
            generic: 1,
            reserve: None,
            source: None,
        };
        let before = game.snapshot();
        let world = FundingState::from_game(&game, &goal);
        compare_moves(&world, &goal);
        let query = FundingKeywordQuery::new(&world);
        assert!(query.evaluation.get().is_none());
        let _ = world.moves_with_keywords(&goal, |r| query.keywords(r));
        assert!(query.evaluation.get().is_some_and(Option::is_some));
        let start = std::time::Instant::now();
        for _ in 0..3 {
            std::hint::black_box(world.moves_with_keywords(&goal, |r| world.keywords(r)));
        }
        let uncached = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..3 {
            std::hint::black_box(world.moves(&goal));
        }
        eprintln!(
            "TMT_FUNDING_KEYWORDS player={p:?} battlefield49 grants12 moves3 uncached={uncached:?} scoped={:?}",
            start.elapsed()
        );
        assert_eq!(game.snapshot(), before);

        // A later query must see a virtual controller change; no derived facts
        // survive beyond the immutable borrow used above.
        let mut changed = world.clone();
        changed
            .resources
            .iter_mut()
            .find(|r| r.id == elves[0])
            .unwrap()
            .object
            .controller = p.other();
        compare_moves(&changed, &goal);
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn funding_keyword_query_removal_and_virtual_departure_match_exhaustive() {
    for p in [Player::P0, Player::P1] {
        let mut game = setup(p);
        let elf = named(&mut game, p, "Llanowar Elves", Zone::Battlefield);
        game.state.objects[elf as usize].summoning_sick = true;
        let source = named(&mut game, p, "Goatnap", Zone::Graveyard);
        game.grant_keyword_effects(&[elf], game.object_ref(source), Keywords::HASTE);
        let aura = named(&mut game, p.other(), "Eaten by Piranhas", Zone::Battlefield);
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(elf));
        game.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let before = game.snapshot();
        let world = FundingState::from_game(&game, &goal);
        compare_moves(&world, &goal);
        assert!(world.moves(&goal).is_empty());
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|r| r.id == aura)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        compare_moves(&departed, &goal);
        assert!(!departed.moves(&goal).is_empty());
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn funding_keyword_query_is_lazy_and_remembers_no_evaluation_needed() {
    for p in [Player::P0, Player::P1] {
        let mut game = setup(p);
        let elf = named(&mut game, p, "Llanowar Elves", Zone::Battlefield);
        named(&mut game, p, "Forest", Zone::Battlefield);
        game.state.objects[elf as usize].summoning_sick = true;
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        let query = FundingKeywordQuery::new(&world);
        let _ = world.moves_with_keywords(&goal, |r| query.keywords(r));
        assert!(query.evaluation.get().is_some_and(Option::is_none));
        compare_moves(&world, &goal);
        let mut mature = world.clone();
        mature
            .resources
            .iter_mut()
            .find(|r| r.id == elf)
            .unwrap()
            .object
            .summoning_sick = false;
        let query = FundingKeywordQuery::new(&mature);
        let _ = mature.moves_with_keywords(&goal, |r| query.keywords(r));
        assert!(
            query.evaluation.get().is_none(),
            "no keyword-dependent readiness check needs an evaluator"
        );
        compare_moves(&mature, &goal);
    }
}
