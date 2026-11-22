//! Cantankerous Captain, Collation Trimatrix, and Crystalline Entity rules.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn finish_with_target(game: &mut Game, target: Option<Target>) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        let actions = game.legal_actions();
        let action = if let Some(target) = target
            && actions.contains(&Action::ChooseTarget { target })
        {
            Action::ChooseTarget { target }
        } else if actions.contains(&Action::FinishStackTargets) {
            Action::FinishStackTargets
        } else {
            actions
                .into_iter()
                .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
                .unwrap_or_else(|| panic!("unhandled {:?}", game.state.pending))
        };
        act(game, action);
    }
    panic!("resolution budget");
}

fn begin_combat(game: &mut Game) {
    game.begin_step(Step::BeginCombat);
    game.advance_replacements(false);
    game.give_priority();
}

#[test]
fn cantankerous_captain_checks_total_power_after_its_boost() {
    let mut threshold = scenario(Player::P0);
    let captain = put(
        &mut threshold,
        Player::P0,
        "Cantankerous Captain",
        Zone::Battlefield,
    );
    put(
        &mut threshold,
        Player::P0,
        "Grizzly Bears",
        Zone::Battlefield,
    );
    put(&mut threshold, Player::P0, "Hill Giant", Zone::Battlefield);
    begin_combat(&mut threshold);
    finish_with_target(&mut threshold, Some(Target::Card(captain)));
    assert_eq!(stats(&threshold, captain), (3, 3));
    assert!(
        threshold
            .object_view(captain)
            .keywords
            .contains(Keywords::MENACE)
    );

    let mut below = scenario(Player::P0);
    let captain = put(
        &mut below,
        Player::P0,
        "Cantankerous Captain",
        Zone::Battlefield,
    );
    put(&mut below, Player::P0, "Grizzly Bears", Zone::Battlefield);
    begin_combat(&mut below);
    finish_with_target(&mut below, Some(Target::Card(captain)));
    assert_eq!(stats(&below, captain), (3, 3));
    assert!(
        !below
            .object_view(captain)
            .keywords
            .contains(Keywords::MENACE)
    );
}

#[test]
fn collation_trimatrix_taps_three_artifact_creatures_as_cost() {
    let mut game = scenario(Player::P0);
    let source = put(
        &mut game,
        Player::P0,
        "Collation Trimatrix",
        Zone::Battlefield,
    );
    let first = put(&mut game, Player::P0, "Assault Drone", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Collective Drone", Zone::Battlefield);
    let noncreature = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    let ordinary = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let noncreature_ref = game.object_ref(noncreature);
    let ordinary_ref = game.object_ref(ordinary);
    reject(
        &mut game,
        Action::ChooseAdditionalCost {
            object: noncreature_ref,
        },
    );
    reject(
        &mut game,
        Action::ChooseAdditionalCost {
            object: ordinary_ref,
        },
    );
    for card in [source, first, second] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    pay(&mut game);
    settle(&mut game);
    for card in [source, first, second] {
        assert!(game.object_view(card).tapped);
    }
    assert!(!game.object_view(noncreature).tapped);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[0].life, 19);
}

#[test]
fn crystalline_entity_only_destroys_nonartifact_creatures_on_cast_entry() {
    let mut cast_entry = scenario(Player::P0);
    let own = put(
        &mut cast_entry,
        Player::P0,
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let opposing = put(&mut cast_entry, Player::P1, "Hill Giant", Zone::Battlefield);
    let artifact = put(
        &mut cast_entry,
        Player::P1,
        "Assault Drone",
        Zone::Battlefield,
    );
    let entity = put(
        &mut cast_entry,
        Player::P0,
        "Crystalline Entity",
        Zone::Hand,
    );
    cast(&mut cast_entry, entity, None);
    settle(&mut cast_entry);
    assert_eq!(cast_entry.object_view(own).zone, Zone::Graveyard);
    assert_eq!(cast_entry.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(cast_entry.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(cast_entry.object_view(entity).zone, Zone::Battlefield);
    assert!(
        cast_entry
            .object_view(entity)
            .keywords
            .contains(Keywords::FLYING)
    );

    let mut uncast_entry = scenario(Player::P0);
    let witness = put(
        &mut uncast_entry,
        Player::P1,
        "Hill Giant",
        Zone::Battlefield,
    );
    let entity = put(
        &mut uncast_entry,
        Player::P0,
        "Crystalline Entity",
        Zone::Hand,
    );
    uncast_entry.move_object(entity, Zone::Battlefield);
    settle(&mut uncast_entry);
    assert_eq!(uncast_entry.object_view(witness).zone, Zone::Battlefield);
}
