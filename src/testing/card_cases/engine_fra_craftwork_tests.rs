//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact pairs of Craftwork Crusher's triggered modes.
use super::*;

#[test]
fn craftwork_nontargeted_pair_remains_legal_without_a_target() {
    {
        let player = Player::P0;
        let mut game = scenario(player);
        let crusher = put(&mut game, player, "Craftwork Crusher", Zone::Hand);
        cast(&mut game, crusher, None);
        for _ in 0..48 {
            if matches!(game.state.pending, Pending::Modes { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        game.move_object(crusher, Zone::Exile);
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 0 })
        );
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 1 })
        );
        assert!(
            game.legal_actions()
                .contains(&Action::ChooseMode { mode: 2 })
        );
        act(&mut game, Action::ChooseMode { mode: 2 });
        settle(&mut game);
    }
}
