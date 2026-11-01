//! The entry predicate must read speculative resources, not the original game.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn lonely_equipment_entry_uses_projected_control_zone_and_actual_token_subtype() {
    for p in [Player::P0, Player::P1] {
        let mut game = setup(p);
        let land = named(&mut game, p, "The Lonely Mountain", Zone::Hand);
        let equipment = named(&mut game, p, "@token/hob-axe", Zone::Battlefield);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let base = FundingState::from_game(&game, &goal);
        let movement = game.entry_move(land, Zone::Battlefield, false);
        assert!(!base.entry_tapped(&movement));
        let mut departed = base.clone();
        departed
            .resources
            .iter_mut()
            .find(|r| r.id == equipment)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(departed.entry_tapped(&movement));
        let mut stolen = base.clone();
        stolen
            .resources
            .iter_mut()
            .find(|r| r.id == equipment)
            .unwrap()
            .object
            .controller = p.other();
        assert!(stolen.entry_tapped(&movement));
        assert!(
            !base.entry_tapped(&movement),
            "child changes must not mutate parent"
        );
        assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(game.object_view(equipment).controller, p);
    }
}
