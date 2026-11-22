//! Dr. McCoy's printed flash and two distinct entry modes.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn bones(game: &mut Game) -> CardId {
    put(game, Player::P0, "Dr. Leonard \"Bones\" McCoy", Zone::Hand)
}

fn mode(game: &mut Game, choice: u16) {
    for _ in 0..24 {
        if game
            .legal_actions()
            .contains(&Action::ChooseMode { mode: choice })
        {
            act(game, Action::ChooseMode { mode: choice });
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Bones entry mode was not offered: {:?}", game.state.pending);
}

fn drive(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && !game.state.pending_triggers.is_empty() {
            game.prepare_triggers();
        }
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Bones resolution budget: {:?}", game.state.pending);
}

#[test]
fn flash_entry_returns_only_an_owned_non_doctor_permanent_card() {
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Graveyard);
    let doctor = put(
        &mut game,
        Player::P0,
        "Dr. Leonard \"Bones\" McCoy",
        Zone::Graveyard,
    );
    let instant = put(&mut game, Player::P0, "Opt", Zone::Graveyard);
    let opponent_card = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let source = bones(&mut game);
    game.state.active = Player::P1;
    game.state.priority = Player::P0;
    assert!(
        game.legal_actions()
            .iter()
            .any(|action| { matches!(action, Action::Cast { card, .. } if *card == source) })
    );
    cast(&mut game, source, None);
    mode(&mut game, 0);
    for invalid in [doctor, instant, opponent_card] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    drive(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(game.keywords(source).contains(Keywords::FLASH));
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
}

#[test]
fn second_entry_mode_watches_up_to_one_other_creature_after_bones_leaves() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let source = bones(&mut game);
    cast(&mut game, source, None);
    mode(&mut game, 1);
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    drive(&mut zero);
    assert!(zero.state.delayed_target_events.is_none());
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drive(&mut game);
    assert_eq!(game.state.delayed_target_events.as_ref().unwrap().len(), 1);
    game.move_object(source, Zone::Graveyard);
    game.move_object(victim, Zone::Graveyard);
    drive(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    game.begin_step(Step::End);
    game.prepare_triggers();
    drive(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).controller, Player::P1);
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 1);
    assert_eq!(zero.object_view(victim).zone, Zone::Battlefield);
}
