use super::*;

#[test]
fn schema183_preserves_both_views_of_unrelated_live_state() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        assert!(prior_schema183_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema183(&encoded).unwrap(), encoded);
        }
        let saved = g.snapshot();
        g.restore(&saved).unwrap();
        assert!(prior_schema183_state_supported(&g.state));
    }
}

#[test]
fn schema183_rejects_real_delayed_trigger_after_registry_expires() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let garruk = Card::from_name("Garruk, Curse Breaker").unwrap();
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(garruk),
            CopyDeadline::EndOfTurn,
        );
        g.state.objects[source as usize].counters.loyalty = 5;
        let attacker = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        // Departure restores the underlying supported Bear Cub definition.
        g.move_object(source, Zone::Exile);
        assert_eq!(
            g.state.objects[source as usize].card,
            Card::from_name("Bear Cub").unwrap()
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        assert!(
            g.state
                .stack
                .iter()
                .chain(&g.state.pending_triggers)
                .chain(&g.state.ordering_triggers)
                .any(
                    |item| item.source.expect("object-backed packet").card == garruk
                        && matches!(item.kind, StackKind::TriggeredAbility { .. })
                )
        );
        // Force the deadline boundary while the already-created trigger remains
        // pending, to audit owned continuation independently from the registry.
        // Normal priority rules would resolve it before the next ordinary turn.
        g.state.active = player;
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
        assert!(!prior_schema183_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema183(&encoded).is_err());
        }
        let saved = g.snapshot();
        g.state.stack.clear();
        g.state.pending_triggers.clear();
        g.state.ordering_triggers.clear();
        assert!(prior_schema183_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema183_state_supported(&g.state));
    }
}
