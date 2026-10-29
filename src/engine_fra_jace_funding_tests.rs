//! Projected funding must retain non-mana Jace witnesses for loyalty guards.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_jace_loyalty_uses_all_witnesses_and_virtual_changes() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        let definition = Card::from_name("Jace, Reality Sculptor").unwrap();
        g.state.objects[source as usize].card = definition;
        g.state.objects[source as usize].counters.loyalty = 12;
        let witness = named(&mut g, player, "The Theorist, Jace Beleren", Zone::Battlefield);
        g.state.objects[witness as usize].counters.loyalty = 13;
        let guard = ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype: "Jace", minimum: 25 };
        let goal = FundingGoal {
            actor: player, required: [0; 6], generic: 0, reserve: None,
            source: Some((g.object_ref(source), 2, Counters::default(), definition, None)),
        };
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert!(world.resources.iter().any(|r| r.id == witness));
        assert!(world.guard_available(guard, source, player));
        let mut departed = world.clone();
        departed.resources.iter_mut().find(|r| r.id == witness).unwrap().object.zone = Zone::Graveyard;
        assert!(!departed.guard_available(guard, source, player));
        let mut stolen = world.clone();
        stolen.resources.iter_mut().find(|r| r.id == witness).unwrap().object.controller = player.other();
        assert!(!stolen.guard_available(guard, source, player));
        let mut reduced = world.clone();
        reduced.resources.iter_mut().find(|r| r.id == witness).unwrap().object.counters.loyalty = 12;
        assert!(!reduced.guard_available(guard, source, player));
        assert_eq!(g.snapshot(), saved);
    }
}
