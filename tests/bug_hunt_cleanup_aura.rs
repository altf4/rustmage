mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn sheltered_by_ghosts_falling_off_grants_cleanup_priority() {
    let mut game = scenario(Player::P0);
    let giant = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let elf = put(&mut game, Player::P1, "Llanowar Elves", Zone::Battlefield);
    let steal = put(&mut game, Player::P0, "Involuntary Employment", Zone::Hand);
    let aura = put(&mut game, Player::P0, "Sheltered by Ghosts", Zone::Hand);

    cast(&mut game, steal, Some(Target::Card(giant)));
    settle(&mut game);
    assert_eq!(game.object_view(giant).controller, Player::P0);

    cast(&mut game, aura, Some(Target::Card(giant)));
    for _ in 0..20 {
        if game.object_view(elf).zone == Zone::Exile {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
                || *action
                    == Action::ChooseTarget {
                        target: Target::Card(elf),
                    }
        });
    }
    assert_eq!(game.object_view(elf).zone, Zone::Exile);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
    let turn = game.state.turn;
    for _ in 0..30 {
        if game.state.step == Step::End {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_eq!(game.state.step, Step::End);
    act(&mut game, Action::Pass);
    let before = game.snapshot();
    let actor = game.actor().unwrap();
    let pass = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(actor, game.revision(), pass).unwrap();
    act(&mut game, Action::Pass);
    assert!(indexed.snapshot() == game.snapshot());
    indexed.restore(&before).unwrap();
    act(&mut indexed, Action::Pass);
    assert!(indexed.snapshot() == game.snapshot());

    assert_eq!(game.object_view(giant).controller, Player::P1);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(game.object_view(elf).zone, Zone::Battlefield);
    assert_eq!(
        (game.state.turn, game.state.step, game.actor()),
        (turn, Step::Cleanup, Some(Player::P0)),
        "the Aura's state-based move should grant priority before the turn advances"
    );

    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!((game.state.turn, game.state.step), (turn + 1, Step::Upkeep));
}
