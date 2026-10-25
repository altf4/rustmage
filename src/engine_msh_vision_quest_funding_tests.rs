//! Actual versus borrowed projected post-entry counter and keyword completion.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn msh_vision_quest_projected_entry_counter_and_postentry_keyword_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, "A.I.M. Synthoids", Zone::Hand);
        let source = named(&mut g, p, "Shock", Zone::Graveyard);
        let mut m = g.entry_move(b, Zone::Battlefield, false);
        m.effect_counter = Some((CounterKind::PlusOne, 4));
        m.keywords_after = Some((g.object_ref(source), Keywords::HASTE));
        let mut actual = g.clone();
        actual.stage_entries(vec![m.clone()], None);
        assert!(!actual.advance_entries());
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let mut visits = 0;
        world.visit_entry_moves(vec![m], true, None, &mut |after| {
            visits += 1;
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(b),
                actual.object_view(b)
            );
            false
        });
        assert_eq!(visits, 1);
        assert!(actual.keywords(b).contains(Keywords::HASTE));
    }
}
