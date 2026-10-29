use super::*;

fn review_jace(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Jace, Reality Sculptor").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}

#[test]
fn jace_threshold_uses_copied_current_type_and_subtype_with_restore() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = review_jace(&mut g, player);
        g.state.objects[source as usize].counters.loyalty = 12;
        let other = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[other as usize].counters.loyalty = 13;
        let guard = ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype: "Jace", minimum: 25 };
        assert!(!g.activation_guard_available(guard, source, player));
        let saved = g.snapshot();
        for _ in 0..2 {
            let reference = g.object_ref(other);
            g.apply_temporary_copy(reference, CopyRecipe::new(Card::from_name("Jace, the Perfected Mind").unwrap()), CopyDeadline::EndOfTurn);
            assert!(g.activation_guard_available(guard, source, player));
            g.apply_temporary_copy(reference, CopyRecipe::new(Card::from_name("Bear Cub").unwrap()), CopyDeadline::EndOfTurn);
            assert!(!g.activation_guard_available(guard, source, player));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_jace_delayed_effect_uses_copy_controller_after_source_leaves() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = review_jace(&mut g, player);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: jace, ability: 1, counter_payment: Counters::default() });
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        settle(&mut g);
        let rows = g.state.delayed_opponent_attack_until_next_turn.as_ref().unwrap();
        assert!(rows.iter().any(|row| row.controller == player));
        assert!(rows.iter().any(|row| row.controller == player.other()));
        g.move_object(jace, Zone::Exile);
        g.state.active = player;
        g.state.priority = player;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(stats(&g, attacker), (-3, 2));
            g.restore(&saved).unwrap();
        }
    }
}
