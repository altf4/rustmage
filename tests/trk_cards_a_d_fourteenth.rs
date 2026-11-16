//! Dahar Master's optional sacrifice and reflexive damage target.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

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
fn dahar_master_sacrifices_only_another_nonland_before_choosing_damage_target() {
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let source = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Dahar Master").unwrap(),
        Zone::Hand,
    );
    cast(&mut game, source, None);
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let checkpoint = game.snapshot();
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    drain(&mut declined);
    assert_eq!(declined.state.players[1].life, 20);
    assert_eq!(declined.object_view(artifact).zone, Zone::Battlefield);

    game.restore(&checkpoint).unwrap();
    let land_ref = game.object_ref(land);
    let source_ref = game.object_ref(source);
    reject(&mut game, Action::Sacrifice { object: land_ref });
    reject(&mut game, Action::Sacrifice { object: source_ref });
    let artifact_ref = game.object_ref(artifact);
    act(
        &mut game,
        Action::Sacrifice {
            object: artifact_ref,
        },
    );
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(Player::P1),
        }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.state.players[1].life, 18);
}
