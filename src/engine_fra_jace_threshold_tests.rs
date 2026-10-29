use super::*;

#[test]
fn jace_loyalty_threshold_counts_only_controlled_current_jace_planeswalkers() {
    let guard = ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast {
        subtype: "Jace", minimum: 25,
    };
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let first = put(&mut g, player, "Jace, the Perfected Mind", Zone::Battlefield);
        let second = put(&mut g, player, "Jace, the Perfected Mind", Zone::Battlefield);
        let other = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Jace, the Perfected Mind", Zone::Battlefield);
        g.state.objects[first as usize].counters.loyalty = 12;
        g.state.objects[second as usize].counters.loyalty = 12;
        g.state.objects[other as usize].counters.loyalty = 100;
        g.state.objects[enemy as usize].counters.loyalty = 100;
        assert!(!g.activation_guard_available(guard, first, player));
        g.state.objects[second as usize].counters.loyalty = 13;
        assert!(g.activation_guard_available(guard, first, player));
        let saved = g.snapshot();
        let second_ref = g.object_ref(second);
        g.change_controller(second_ref, player.other());
        assert!(!g.activation_guard_available(guard, first, player));
        g.restore(&saved).unwrap();
        assert!(g.activation_guard_available(guard, first, player));
    }
}
