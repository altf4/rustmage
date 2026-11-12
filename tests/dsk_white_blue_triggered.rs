//! Card-level scenarios for Duskmourn's white and blue triggered effects.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn splitskin_doll_draws_first_and_checks_other_controlled_creatures_at_resolution() {
    let mut game = scenario(PLAYER);
    let doll = put(&mut game, PLAYER, "Splitskin Doll", Zone::Hand);
    let small = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let growth = put(&mut game, PLAYER, "Giant Growth", Zone::Hand);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    cast(&mut game, doll, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, doll), (2, 1));
    assert_eq!(game.object_view(top).zone, Zone::Library);

    let mut with_small_creature = game.clone();
    settle(&mut with_small_creature);
    assert_eq!(with_small_creature.object_view(top).zone, Zone::Hand);
    assert_eq!(with_small_creature.object_view(growth).zone, Zone::Hand);

    cast(&mut game, growth, Some(Target::Card(small)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, small), (5, 5));
    pass_pair(&mut game);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { player: PLAYER, .. }
    ));
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert!(
        game.legal_actions()
            .contains(&Action::Discard { card: top })
    );
    assert!(
        game.observe(OPPONENT)
            .objects
            .iter()
            .all(|object| object.id != top)
    );
    act(&mut game, Action::Discard { card: top });
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}
