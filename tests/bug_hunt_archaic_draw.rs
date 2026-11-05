mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn archaic_discards_after_its_draw_replacement_choice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bard = put(&mut game, player, "Bard, King of Dale", Zone::Battlefield);
    let riddler = put(&mut game, player, "Quantum Riddler", Zone::Battlefield);
    let archaic = put(&mut game, player, "Transcendent Archaic", Zone::Hand);
    game.state.players[player.index()].mana = [0, 7, 0, 0, 0, 0];
    cast(&mut game, archaic, None);
    for _ in 0..16 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(&mut game, Action::ChooseOptional { accept: true });

    let choose_replacement = |card| Action::ChooseDrawReplacement {
        source: game.object_ref(card),
    };
    let bard_first = choose_replacement(bard);
    let riddler_first = choose_replacement(riddler);
    let actions = game.legal_actions();
    assert!(actions.contains(&bard_first));
    assert!(actions.contains(&riddler_first));

    let checkpoint = game.snapshot();
    let index = actions
        .iter()
        .position(|action| *action == riddler_first)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, riddler_first.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, riddler_first);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
    assert!(
        matches!(game.state.pending, Pending::EffectDiscard { player: p, remaining: 2 } if p == player),
        "after drawing four, the Archaic must require two discards. Actual {:?}",
        game.state.pending
    );
    for remaining in [1, 0] {
        choose(&mut game, |action| matches!(action, Action::Discard { .. }));
        if remaining == 1 {
            assert!(matches!(
                game.state.pending,
                Pending::EffectDiscard { player: p, remaining: 1 } if p == player
            ));
        }
    }
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.state.pending, Pending::Priority);
}
