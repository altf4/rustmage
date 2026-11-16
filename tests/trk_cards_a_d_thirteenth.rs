//! Dathon and Picard's distinct modal targets and opponent choice.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn spell(game: &mut Game) -> CardId {
    spawn(
        game,
        Player::P0,
        Card::from_name("Dathon and Picard at El-Adrel").unwrap(),
        Zone::Hand,
    )
}
fn drain(game: &mut Game) {
    for _ in 0..50 {
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
    panic!("resolution budget");
}

#[test]
fn dathon_and_picard_pump_mode_grants_stats_and_trample() {
    let mut game = scenario(Player::P0);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let card = spell(&mut game);
    act(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::ChooseMode { mode: 0 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, creature), (5, 5));
    assert!(game.keywords(creature).contains(Keywords::TRAMPLE));
}

#[test]
fn dathon_and_picard_sacrifice_mode_excludes_tokens_and_creatures() {
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P1, "Tricorder", Zone::Battlefield);
    let token = spawn(&mut game, Player::P1, Card(5296), Zone::Battlefield);
    let creature = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let card = spell(&mut game);
    act(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.actor(), Some(Player::P1));
    let token_ref = game.object_ref(token);
    let creature_ref = game.object_ref(creature);
    reject(&mut game, Action::Sacrifice { object: token_ref });
    reject(
        &mut game,
        Action::Sacrifice {
            object: creature_ref,
        },
    );
    let artifact_ref = game.object_ref(artifact);
    act(
        &mut game,
        Action::Sacrifice {
            object: artifact_ref,
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
}
