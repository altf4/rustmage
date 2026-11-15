//! Final T–Z cards with source-exact permissions and entry obligations.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn top_casts(game: &Game, card: CardId) -> Vec<Action> {
    game.legal_actions().into_iter().filter(|action| matches!(action,
        Action::AnnounceLibraryTop { object, method: CastMethod::Normal, .. }
            if *object == game.object_ref(card)
    )).collect()
}

#[test]
fn zefram_looks_privately_casts_one_top_artifact_and_adds_spacecraft_charge_counters() {
    let mut game = scenario(Player::P0);
    let zefram = put(&mut game, Player::P0, "Zefram Cochrane, Warp Pioneer", Zone::Battlefield);
    let next = put(&mut game, Player::P0, "Tricorder", Zone::Library);
    let first = put(&mut game, Player::P0, "Short Bow", Zone::Library);
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object,
        game.object_ref(first));
    assert!(game.observe(Player::P1).known_library_top[0].is_none());
    let action = top_casts(&game, first).into_iter().find(|action| matches!(action,
        Action::AnnounceLibraryTop { permission, .. }
            if permission.source == game.object_ref(zefram)
    )).expect("Zefram permits one artifact cast from top");
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert!(top_casts(&game, next).is_empty());
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object,
        game.object_ref(next));
    assert!(game.observe(Player::P1).known_library_top[0].is_none());

    let spacecraft = put(&mut game, Player::P0, "Romulan Bird-of-Prey", Zone::Hand);
    cast(&mut game, spacecraft, None);
    settle(&mut game);
    assert_eq!(game.object_view(spacecraft).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(spacecraft, CounterKind::Charge), 2);
    assert_eq!(game.counter_count(first, CounterKind::Charge), 0);

    game.move_object(zefram, Zone::Graveyard);
    assert!(game.observe(Player::P0).known_library_top[0].is_none());
    let later = put(&mut game, Player::P0, "Romulan Bird-of-Prey", Zone::Hand);
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(game.counter_count(later, CounterKind::Charge), 0);
}
