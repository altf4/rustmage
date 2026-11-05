mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn this_turn_delayed_triggers_observe_cleanup_actions_until_the_turn_advances() {
    let p = Player::P0;
    let mut game = scenario(p);
    let sentry = put(&mut game, p, "Magnigoth Sentry", Zone::Battlefield);
    let survivor = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let mountain = put(&mut game, p, "Mountain", Zone::Battlefield);
    let bolt = put(&mut game, p, "Lightning Bolt", Zone::Hand);
    let turn = game.state.turn;
    for (name, target) in [
        ("Giant Growth", sentry),
        ("Blight Rot", sentry),
        ("Desperate Measures", sentry),
        ("Desperate Measures", survivor),
    ] {
        let spell = put(&mut game, p, name, Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
    }
    assert_eq!(stats(&game, sentry).1, 2);
    for _ in 0..32 {
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
    let before = game.hand_count(p);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.object_view(sentry).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.hand_count(p), before + 2);

    // An untriggered watch survives this cleanup priority, but not a turn change.
    let mut next_turn = game.clone();
    act(&mut next_turn, Action::Pass);
    act(&mut next_turn, Action::Pass);
    assert_eq!(next_turn.state.turn, turn + 1);
    assert!(next_turn.state.delayed_target_events.is_none());

    act(&mut game, Action::TapMana { card: mountain });
    cast(&mut game, bolt, Some(Target::Card(survivor)));
    settle(&mut game);
    assert_eq!(game.object_view(survivor).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(p), before + 3);
    assert_eq!((game.state.turn, game.state.step), (turn, Step::Cleanup));
}
