//! Replacement-to-entry handoffs have one owner for the suspended program.
mod support;

use rustmage as _;
use rustmage_core::cards::CounterKind;
use rustmage_core::engine::*;
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn goblin_discard_and_choice_free_treasure_entry_finish_the_program_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let goblin = put(
        &mut game,
        player,
        "Ultimate Green Goblin",
        Zone::Battlefield,
    );
    put(&mut game, player.other(), "Spider-Punk", Zone::Battlefield);
    let discard = put(&mut game, player, "Swamp", Zone::Hand);
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, goblin);
    resolve_top(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectDiscard { .. }));

    let action = Action::Discard { card: discard };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let before = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].cards_discarded_this_turn,
        1
    );
    assert!(game.state.resolving.is_none());
    assert!(game.state.entry_batch.is_none());
    assert!(game.state.replacements.is_none());
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(
        game.observe(player)
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.has_subtype("Treasure")
            })
            .count(),
        1,
    );
}

#[test]
fn a_real_riot_choice_resumes_the_rest_of_wall_crawls_program_after_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Spider-Punk", Zone::Battlefield);
    let wall = put(&mut game, player, "Wall Crawl", Zone::Hand);
    cast(&mut game, wall, None);
    resolve_top(&mut game);
    resolve_top(&mut game);
    assert!(matches!(game.state.pending, Pending::EntryRiot { .. }));
    assert_eq!(game.state.players[player.index()].life, 20);
    assert!(game.state.resolving.is_some());

    act(&mut game, Action::ChooseEntryRiot { counter: true });
    let spider = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Spider")
        })
        .unwrap();
    assert_eq!(game.counter_count(spider.id, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert!(game.state.resolving.is_none());
    assert!(game.state.entry_batch.is_none());
    assert_eq!(game.state.pending, Pending::Priority);
}
