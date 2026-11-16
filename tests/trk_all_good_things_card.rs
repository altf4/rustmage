//! The printed Saga draws, links exact hand exiles, and checks their shared type.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drive(game: &mut Game, until_choice: bool) {
    for _ in 0..48 {
        if until_choice && game.state.pending == (Pending::EffectChoice { player: Player::P0 }) {
            return;
        }
        if game.state.pending == Pending::Priority && !game.state.pending_triggers.is_empty() {
            game.prepare_triggers();
        }
        if !until_choice
            && game.state.pending == Pending::Priority
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
    panic!(
        "All Good Things chapter did not settle: {:?}",
        game.state.pending
    );
}

fn select_hand_exile(game: &mut Game, card: CardId) {
    drive(game, true);
    assert!(game.observe(Player::P1).effect_choice.is_none());
    let object = game.object_ref(card);
    act(game, Action::SelectResolutionExile { object });
    drive(game, false);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
}

fn next_chapter(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
}

#[test]
fn chapter_draws_and_exiles_then_fourth_checks_all_linked_card_types() {
    let mut game = scenario(Player::P0);
    let saga = put(&mut game, Player::P0, "All Good Things...", Zone::Hand);
    let first = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    let second = put(&mut game, Player::P0, "Adaptive Automaton", Zone::Hand);
    let third = put(&mut game, Player::P0, "Gingerbrute", Zone::Hand);
    let mismatch = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    let library_before = game.state.players[0].library.len();
    cast(&mut game, saga, None);

    for (chapter, selected) in [(1, first), (2, second)] {
        if chapter != 1 {
            next_chapter(&mut game);
        }
        drive(&mut game, true);
        assert_eq!(
            game.state.players[0].library.len(),
            library_before - chapter * 2
        );
        select_hand_exile(&mut game, selected);
        assert_eq!(game.counter_count(saga, CounterKind::Lore), chapter as u64);
    }

    next_chapter(&mut game);
    drive(&mut game, true);
    assert_eq!(game.state.players[0].library.len(), library_before - 6);
    let mut no_common_type = game.clone();
    select_hand_exile(&mut no_common_type, mismatch);
    select_hand_exile(&mut game, third);
    assert_eq!(
        game.linked_exile_for_origin(game.object_ref(saga), None)
            .len(),
        3
    );

    let life = game.state.players[0].life;
    next_chapter(&mut game);
    drive(&mut game, false);
    assert_eq!(game.state.players[0].life, life + 10);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);

    let life = no_common_type.state.players[0].life;
    next_chapter(&mut no_common_type);
    drive(&mut no_common_type, false);
    assert_eq!(no_common_type.state.players[0].life, life);
    assert_eq!(no_common_type.object_view(saga).zone, Zone::Graveyard);
}
