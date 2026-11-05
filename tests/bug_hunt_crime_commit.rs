mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn a_bargain_that_returns_control_before_cast_completion_commits_a_crime() {
    let p = Player::P0;
    let mut game = scenario(p);
    let giant = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let engine = put(&mut game, p, "Possession Engine", Zone::Hand);
    let oko = put(&mut game, p, "Oko, the Ringleader", Zone::Hand);
    let grapple = put(&mut game, p, "Candy Grapple", Zone::Hand);

    cast(&mut game, engine, None);
    for _ in 0..20 {
        if game.object_view(giant).controller == p {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || *a
                    == Action::ChooseTarget {
                        target: Target::Card(giant),
                    }
        });
    }
    assert_eq!(game.object_view(giant).controller, p);
    assert!(game.state.players[p.index()].committed_crime_this_turn);
    let next_turn = game.state.turn + 2;
    for _ in 0..90 {
        if game.state.turn == next_turn && game.state.step == Step::Main1 {
            break;
        }
        choose(&mut game, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_eq!(
        (game.state.turn, game.state.step, game.actor()),
        (next_turn, Step::Main1, Some(p))
    );
    assert!(!game.state.players[p.index()].committed_crime_this_turn);
    game.state.players[p.index()].mana = [0, 4, 2, 0, 4, 0];
    cast(&mut game, oko, None);
    settle(&mut game);

    game.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: grapple });
    let sacrifice = Some(game.object_ref(engine));
    act(&mut game, Action::ChooseBargain { sacrifice });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(giant),
        },
    );
    assert_eq!(game.object_view(giant).controller, p);
    pay(&mut game);
    assert_eq!(game.object_view(grapple).zone, Zone::Stack);
    assert_eq!(game.object_view(engine).zone, Zone::Graveyard);
    assert_eq!(game.object_view(giant).controller, p.other());
    let crime = game.state.players[p.index()].committed_crime_this_turn;
    settle(&mut game);

    act(
        &mut game,
        Action::AnnounceActivate {
            card: oko,
            ability: 0,
            counter_payment: Default::default(),
        },
    );
    if game
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let mut discards = 0;
    while matches!(game.state.pending, Pending::EffectDiscard { .. }) {
        choose(&mut game, |a| matches!(a, Action::Discard { .. }));
        discards += 1;
    }
    assert_eq!(
        (crime, discards),
        (true, 1),
        "crime must use control when the spell becomes cast"
    );
}
