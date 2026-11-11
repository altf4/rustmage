//! The opposing-hand guard samples only the other seat's current hand count.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn opponent_hand_qualifies_independently_of_the_sources_controller() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Hunter's Talent", Zone::Battlefield);
        for _ in 0..3 {
            put(&mut game, player, "Forest", Zone::Hand);
        }
        let guard = StepGuard::Expression {
            value: StatValue::OpponentHandSize,
            minimum: None,
            maximum: Some(1),
        };
        let reference = game.object_ref(source);
        assert!(game.step_guard(guard, player, reference, false));
        let first = put(&mut game, player.other(), "Forest", Zone::Hand);
        assert!(game.step_guard(guard, player, reference, false));
        put(&mut game, player.other(), "Forest", Zone::Hand);
        assert!(!game.step_guard(guard, player, reference, false));
        game.move_object(first, Zone::Exile);
        assert!(game.step_guard(guard, player, reference, false));
    }
}
