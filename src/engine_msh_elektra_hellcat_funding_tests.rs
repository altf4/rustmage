//! Exact returned-recipient layers in the borrowed entry world.
use super::*;
use crate::engine::assurance::big_helpers::setup;
use crate::engine::msh_elektra_hellcat_tests::{packet, paid, until};
const HELLCAT: &str = "Hellcat, Undying Vigilante";
#[test]
fn msh_elektra_hellcat_projected_entry_matches_actual_return_modification() {
    use super::ability_world::CharacteristicWorld;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        packet(&mut g, p, "Murder", &[Target::Card(h)]);
        until(&mut g, |g| g.state.stack.iter().any(|s| s.card == h));
        let source = g.state.stack.iter().find(|s| s.card == h).unwrap().object;
        let mut m = g.entry_move(h, Zone::Battlefield, false);
        m.controller = p;
        m.effect_counter = Some((CounterKind::PlusOne, 1));
        m.blank_with_haste_after = Some(source);
        let before = g.snapshot();
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
                characteristic_query::CharacteristicQuery::in_world(after).object_view(h),
                actual.object_view(h)
            );
            assert!(!after.printed_abilities_available(h));
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
