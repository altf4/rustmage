//! Failed multicolor funding must not enumerate unrelated basic-land subsets.
use super::super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

#[test]
fn funding_symmetry_weather_charge_and_many_forests_have_linear_failed_search() {
    for (player, forest_count) in [Player::P0, Player::P1]
        .into_iter()
        .flat_map(|player| [6, 12, 18].map(|count| (player, count)))
    {
        let mut game = setup(player);
        let weather = named(&mut game, player, "Weather Maker", Zone::Hand);
        cast(&mut game, weather, None, &[]);
        settle(&mut game);
        let first = named(&mut game, player, "Forest", Zone::Hand);
        checked(&mut game, Action::PlayLand { card: first });
        settle(&mut game);
        assert_eq!(game.counter_count(weather, CounterKind::Charge), 1);
        let mut forests = vec![first];
        for _ in 1..forest_count {
            forests.push(named(&mut game, player, "Forest", Zone::Battlefield));
        }
        let pizza = named(&mut game, player, "Everything Pizza", Zone::Battlefield);
        game.state.players[player.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: player,
            required: [1, 1, 1, 1, 1, 0],
            generic: 2,
            reserve: None,
            source: None,
        };
        let before = game.snapshot();
        let view = game.observe(player);
        let world = FundingState::from_game(&game, &goal);
        // Independent color maxima deliberately overestimate the same flexible
        // source. This exercises failed DFS, rather than an immediate bound.
        assert!(world.optimistic(&goal));
        // The land played through the real action is newly entered; the other
        // fixture lands are mature. Sickness remains in ResourceKey, so there
        // are two tap-equivalence classes of sizes 1 and N-1. Swapping taps
        // within a class preserves the key, while crossing classes does not.
        assert_ne!(
            game.state.objects[first as usize].summoning_sick,
            game.state.objects[forests[1] as usize].summoning_sick,
        );
        assert!(
            forests[1..]
                .iter()
                .all(|&id| game.state.objects[id as usize].summoning_sick
                    == game.state.objects[forests[1] as usize].summoning_sick)
        );
        let tap_key = |id| {
            let mut next = world.clone();
            next.resources
                .iter_mut()
                .find(|r| r.id == id)
                .unwrap()
                .object
                .tapped = true;
            next.key(&goal)
        };
        assert!(tap_key(forests[1]) == tap_key(forests[2]));
        assert!(tap_key(first) != tap_key(forests[1]));
        let class_combinations = (1 + 1) * ((forest_count - 1) + 1);
        let mut visited = HashSet::new();
        let start = std::time::Instant::now();
        assert!(!world.reachable(&goal, &mut visited));
        let elapsed = start.elapsed();
        assert!(
            visited.len() <= class_combinations,
            "visited {} states",
            visited.len()
        );
        eprintln!(
            "TMT_SYMMETRY forests={forest_count} visited={} elapsed={elapsed:?}",
            visited.len()
        );
        assert!(!game.can_fund(goal));
        assert!(!game.legal_actions().iter().any(|a| matches!(a,
            Action::AnnounceActivate { card, .. } if *card == pizza)));
        assert_eq!(game.snapshot(), before);
        assert_eq!(
            serde_json::to_value(game.observe(player)).unwrap(),
            serde_json::to_value(view).unwrap()
        );
    }
}

#[test]
fn funding_symmetry_named_counter_basic_lands_keep_exact_identity() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let lands: Vec<_> = (0..4)
            .map(|_| named(&mut game, player, "Forest", Zone::Battlefield))
            .collect();
        game.add_counter(lands[0], CounterKind::Charge, 1);
        game.add_counter(lands[1], CounterKind::Charge, 1);
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        let tap = |id| {
            let mut next = world.clone();
            next.resources
                .iter_mut()
                .find(|r| r.id == id)
                .unwrap()
                .object
                .tapped = true;
            next.key(&goal)
        };
        assert!(
            tap(lands[0]) != tap(lands[1]),
            "equal named-counter counts still refer to distinct exact rows"
        );
        assert!(tap(lands[0]) != tap(lands[2]));
        assert!(
            tap(lands[2]) == tap(lands[3]),
            "unrelated named rows must not distinguish plain basics"
        );
    }
}
