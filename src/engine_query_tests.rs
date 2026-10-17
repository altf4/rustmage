//! Compact query profiling and coherence regressions, without a game-playing policy.
use super::ability_timeline_tests::{cast, named, settle, setup};
use super::*;

fn query_fixture() -> (Game, Game) {
    let p = Player::P0;
    let mut game = setup(p);
    let mut target = None;
    for player in [Player::P0, Player::P1] {
        for name in [
            "Bear Cub",
            "Llanowar Elves",
            "Forensic Researcher",
            "Forest",
            "Island",
        ] {
            for _ in 0..3 {
                let id = named(&mut game, player, name, Zone::Battlefield);
                if player == p && name == "Llanowar Elves" {
                    target = Some(id);
                }
            }
        }
    }
    let baseline = game.clone();
    let aura = named(&mut game, p, "Stop Cold", Zone::Hand);
    cast(&mut game, aura, None, &[Target::Card(target.unwrap())]);
    settle(&mut game);
    (baseline, game)
}

#[test]
#[ignore = "compact optimized profiling workload; run explicitly with --release --ignored"]
fn query_profile_compact_characteristic_workload() {
    let (baseline, active) = query_fixture();
    for (label, game) in [("no_removal", &baseline), ("active_removal", &active)] {
        let before = game.snapshot();
        for _ in 0..2 {
            std::hint::black_box(game.observe(Player::P0));
        }
        let start = std::time::Instant::now();
        let iterations = 24;
        for _ in 0..iterations {
            std::hint::black_box(game.observe(Player::P0));
        }
        println!(
            "QUERY_PROFILE label={label} iterations={iterations} seconds={:.9} logical_bytes={}",
            start.elapsed().as_secs_f64(),
            game.state_bytes()
        );
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn query_matches_fresh_views_and_owned_restore_after_removal_leaves() {
    let (baseline, active) = query_fixture();
    for (has_removal, game) in [(false, &baseline), (true, &active)] {
        let before = game.snapshot();
        let query = game.characteristic_query();
        assert_eq!(
            game.state
                .objects
                .iter()
                .enumerate()
                .any(|(id, _)| query.ability_availability(id as CardId).is_some()),
            has_removal
        );
        for id in 0..game.state.objects.len() as CardId {
            assert_eq!(
                serde_json::to_value(query.object_view(id)).unwrap(),
                serde_json::to_value(game.object_view(id)).unwrap(),
                "shared versus fresh query for {id}"
            );
            assert_eq!(query.stats(id), game.stats(id));
        }
        assert_eq!(game.snapshot(), before);
    }
    let frozen = active.snapshot();
    let expected = [active.observe(Player::P0), active.observe(Player::P1)];
    let mut branch = {
        let parent = active.clone();
        parent.clone()
    };
    let aura = branch
        .state
        .objects
        .iter()
        .position(|o| {
            o.card == Card::from_name("Stop Cold").unwrap() && o.zone == Zone::Battlefield
        })
        .unwrap() as CardId;
    branch.move_object(aura, Zone::Graveyard);
    assert!(!branch.state.objects.iter().enumerate().any(|(id, _)| {
        branch
            .characteristic_query()
            .ability_availability(id as CardId)
            .is_some()
    }));
    let elf = branch
        .state
        .objects
        .iter()
        .position(|o| {
            o.card == Card::from_name("Llanowar Elves").unwrap() && o.zone == Zone::Battlefield
        })
        .unwrap() as CardId;
    assert!(branch.printed_abilities_available(elf));
    branch.restore(&frozen).unwrap();
    assert!(branch.state.objects.iter().enumerate().any(|(id, _)| {
        branch
            .characteristic_query()
            .ability_availability(id as CardId)
            .is_some()
    }));
    for player in [Player::P0, Player::P1] {
        let restored = branch.observe(player);
        let mut expected = serde_json::to_value(&expected[player.index()]).unwrap();
        // A successful restore deliberately invalidates external revision handles.
        expected["revision"] = serde_json::to_value(restored.revision).unwrap();
        assert_eq!(serde_json::to_value(restored).unwrap(), expected);
    }
    assert_eq!(active.snapshot(), frozen);
}
