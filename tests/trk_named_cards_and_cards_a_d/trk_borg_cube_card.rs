//! Borg Cube's affinity, linked fallback, Station, and stationed attack trigger.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn until(game: &mut Game, action: &Action) {
    for _ in 0..64 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |candidate| {
            matches!(candidate, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Borg Cube choice was not reached: {action:?}");
}

#[test]
fn borg_cube_paid_affinity_exiles_then_stations_and_assimilates_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let second = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let opposing_artifact_creature =
        put(&mut game, player.other(), "Gingerbrute", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let cube = put(&mut game, player, "Borg Cube", Zone::Hand);
    assert_eq!(game.object_view(cube).card.fixed_mana_value(), 7);
    // Only the two controlled artifact creatures reduce the five generic mana.
    game.state.players[player.index()].mana = [1, 0, 1, 0, 0, 3];
    cast(&mut game, cube, None);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let target = Action::ChooseTarget {
        target: Target::Card(victim),
    };
    until(&mut game, &target);
    act(&mut game, target);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(cube).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert!(!game.is_creature(cube));
    assert!(!game.keywords(first).contains(Keywords::FLYING));
    let exiled = game.object_ref(victim);

    game.set_counters(cube, Counters::of(CounterKind::Charge, 11));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: cube,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let donor = game.object_ref(first);
    act(&mut game, Action::ChooseAdditionalCost { object: donor });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(first).tapped);
    assert_eq!(game.counter_count(cube, CounterKind::Charge), 12);
    assert!(game.is_creature(cube));
    assert_eq!(stats(&game, cube), (6, 6));
    for id in [cube, first, second] {
        assert!(game.keywords(id).contains(Keywords::FLYING));
    }
    for id in [noncreature, opposing_artifact_creature] {
        assert!(!game.keywords(id).contains(Keywords::FLYING));
    }
    game.state.step = Step::BeginCombat;
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: cube,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );

    game.state.objects[cube as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: cube });
    act(&mut game, Action::FinishAttackers);
    until(&mut game, &Action::FinishTargets);
    act(&mut game, Action::FinishTargets);
    let choose_linked = Action::ChooseLinkedExile { object: exiled };
    until(&mut game, &choose_linked);
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == choose_linked)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, choose_linked.clone());
    settle(&mut game);
    settle(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, choose_linked);
    settle(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).controller, player);
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(victim).kinds & (1 | 16), 1 | 16);
    assert!(game.has_subtype(victim, "Borg"));
    assert!(!game.has_subtype(victim, "Giant"));
    assert!(game.keywords(victim).contains(Keywords::FLYING));
}

#[test]
fn borg_cube_affinity_still_requires_both_colored_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    for _ in 0..6 {
        put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    }
    let cube = put(&mut game, player, "Borg Cube", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Cast { card, .. } if *card == cube)),
        "affinity only reduces generic mana",
    );
    game.state.players[player.index()].mana[0] = 1;
    cast(&mut game, cube, None);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
