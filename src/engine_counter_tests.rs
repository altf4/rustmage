use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
use crate::CounterKind;

fn named(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}
fn enter(game: &mut Game, name: &str) -> CardId {
    let id = named(game, name, Zone::Hand);
    game.move_object(id, Zone::Battlefield);
    game.state_based_actions();
    game.prepare_triggers();
    id
}

#[test]
fn targeted_counter_is_permanent_snapshot_owned_and_resets_on_zone_change() {
    let mut game = scenario();
    let creature = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    enter(&mut game, "Ironpaw Aspirant");
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    let before = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).power, 3);
    assert_eq!(game.object_view(creature).counters.plus_one, 1);
    let after = game.snapshot();
    game.restore(&before);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
    // Cleanup removes temporary growth, but counters remain.
    game.state.objects[creature as usize].growth = 2;
    game.finish_cleanup();
    assert_eq!(game.object_view(creature).power, 3);
    game.move_object(creature, Zone::Hand);
    assert_eq!(game.object_view(creature).counters, Counters::default());
}

#[test]
fn negative_counters_kill_and_opposing_pairs_cancel() {
    let mut game = scenario();
    let creature = spawn(&mut game, Player::P1, Card::HillGiant, Zone::Battlefield);
    game.state.objects[creature as usize]
        .counters
        .add(CounterKind::PlusOne, 2);
    game.state.objects[creature as usize]
        .counters
        .add(CounterKind::MinusOne, 1);
    game.state_based_actions();
    assert_eq!(game.object_view(creature).counters.plus_one, 1);
    assert_eq!(game.object_view(creature).counters.minus_one, 0);
    let rot = named(&mut game, "Blight Rot", Zone::Hand);
    game.state.players[0].mana = [2, 0, 1, 0, 0];
    cast(&mut game, rot, Some(Target::Card(creature)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(creature).counters, Counters::default());
}

#[test]
fn stun_replaces_untap_attempts_including_untap_step_but_not_already_untapped() {
    let mut game = scenario();
    let creature = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    enter(&mut game, "Utrom Scientists");
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    pass_pair(&mut game);
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.object_view(creature).counters.stun, 1);
    let before = game.snapshot();
    game.begin_turn();
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.object_view(creature).counters.stun, 0);
    let after = game.snapshot();
    game.restore(&before);
    game.begin_turn();
    assert_eq!(game.snapshot(), after);
    game.begin_turn();
    assert!(!game.object_view(creature).tapped);
    game.state.objects[creature as usize].counters.stun = 2;
    game.begin_turn();
    assert_eq!(game.object_view(creature).counters.stun, 2);
    game.state.objects[creature as usize].tapped = true;
    let untap = named(&mut game, "Thistledown Players", Zone::Hand);
    game.move_object(untap, Zone::Battlefield);
    game.attack_triggers(&game.object_view(untap));
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    pass_pair(&mut game);
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.object_view(creature).counters.stun, 1);
}

#[test]
fn source_counter_does_not_follow_a_new_incarnation() {
    let mut game = scenario();
    let chocobo = enter(&mut game, "Sazh's Chocobo");
    let land = named(&mut game, "Forest", Zone::Hand);
    game.move_object(land, Zone::Battlefield);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(chocobo, Zone::Hand);
    game.move_object(chocobo, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.object_view(chocobo).counters.plus_one, 0);
    game.move_object(land, Zone::Hand);
    game.move_object(land, Zone::Battlefield);
    game.prepare_triggers();
    pass_pair(&mut game);
    assert_eq!(game.object_view(chocobo).counters.plus_one, 1);
}

#[test]
fn cleanup_death_from_counter_preserves_active_player_for_extra_priority() {
    let mut game = scenario();
    let boggart = named(&mut game, "Bile-Vial Boggart", Zone::Battlefield);
    let target = spawn(&mut game, Player::P1, Card::HillGiant, Zone::Battlefield);
    let toughness = game.object_view(boggart).toughness;
    game.state.objects[boggart as usize].counters.minus_one = toughness as u32;
    game.state.objects[boggart as usize].growth = 1;
    game.state.step = Step::Cleanup;
    game.finish_cleanup();
    game.prepare_triggers();
    assert_eq!(game.state.active, Player::P0);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.object_view(boggart).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.object_view(target).counters.minus_one, 1);
    pass_pair(&mut game);
    assert_eq!(game.state.active, Player::P1);
}

#[test]
fn repeated_cleanup_discards_after_death_trigger_draw_and_replays() {
    let mut game = scenario();
    let sentinel = named(&mut game, "Summit Sentinel", Zone::Battlefield);
    for _ in 0..7 {
        named(&mut game, "Forest", Zone::Hand);
    }
    game.state.objects[sentinel as usize].counters.minus_one = 3;
    game.state.objects[sentinel as usize].growth = 1;
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.active, Player::P0);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game); // Sentinel's death trigger draws the eighth card.
    assert_eq!(game.hand_count(Player::P0), 8);
    let before = game.snapshot();
    pass_pair(&mut game); // A full second cleanup must discard.
    assert_eq!(game.state.pending, Pending::Discard);
    assert_eq!(game.state.active, Player::P0);
    assert!(!game.legal_actions().contains(&Action::Pass));
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    let choice = game.snapshot();
    act(&mut game, discard.clone());
    assert_eq!(game.hand_count(Player::P0), 7);
    assert_eq!(game.state.active, Player::P1);
    let after = game.snapshot();
    game.restore(&choice);
    act(&mut game, discard);
    assert_eq!(game.snapshot(), after);
    game.restore(&before);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), choice);
}
