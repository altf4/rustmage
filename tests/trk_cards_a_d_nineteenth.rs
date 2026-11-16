//! De-evolve's Aura layers and threshold on each upkeep.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        let actions = game.legal_actions();
        let action = actions
            .iter()
            .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
            .cloned()
            .unwrap_or_else(|| panic!(
                "drain pending {:?}, active {:?}, priority {:?}, triggers {}, stack {}, actions {:?}",
                game.state.pending, game.state.active, game.state.priority,
                game.state.pending_triggers.len(), game.state.stack.len(), actions,
            ));
        act(game, action);
    }
    panic!("resolution budget");
}

fn upkeep(game: &mut Game, active: Player) {
    game.state.active = active;
    game.begin_step(Step::Upkeep);
    game.advance_replacements(false);
    game.give_priority();
}

#[test]
fn de_evolve_replaces_only_creature_subtype_and_weakens_the_attachment() {
    let mut game = scenario(Player::P0);
    let borg = put(&mut game, Player::P1, "Assault Drone", Zone::Battlefield);
    let aura = put(&mut game, Player::P0, "De-evolve", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(borg)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
    assert_eq!(game.object_view(borg).kinds & (1 | 16), 1 | 16);
    assert_eq!(stats(&game, borg), (-1, 4));
    assert!(game.has_subtype(borg, "Salamander"));
    assert!(!game.has_subtype(borg, "Borg"));
    assert!(
        game.object_view(borg)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
}

#[test]
fn de_evolve_threshold_checks_each_upkeep_and_rechecks_on_resolution() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, Player::P0, "De-evolve", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(victim)));
    settle(&mut game);
    for _ in 0..6 {
        put(&mut game, Player::P0, "Shock", Zone::Graveyard);
    }
    upkeep(&mut game, Player::P1);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);

    let seventh = put(&mut game, Player::P0, "Shock", Zone::Graveyard);
    upkeep(&mut game, Player::P1);
    assert!(
        !game.state.pending_triggers.is_empty(),
        "seven own graveyard cards did not trigger: source {:?}, actions {:?}",
        game.object_view(aura),
        game.legal_actions()
    );
    game.move_object(seventh, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);

    game.move_object(seventh, Zone::Graveyard);
    upkeep(&mut game, Player::P1);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
}
