use super::*;

fn garruk_schedule(g: &mut Game, player: Player) -> CardId {
    let source = put(g, player, "Garruk, Curse Breaker", Zone::Battlefield);
    g.state.objects[source as usize].counters.loyalty = 5;
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    settle(g);
    source
}
fn garruk_attack(g: &mut Game, creature: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: creature });
    act(g, Action::FinishAttackers);
    g.prepare_triggers();
}

#[test]
fn copied_delayed_attack_trigger_preserves_exact_attacker_subjects() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = garruk_schedule(&mut g, player);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        garruk_attack(&mut g, attacker);
        let original = g
            .state
            .stack
            .iter()
            .find(|item| {
                item.card == source && matches!(item.kind, StackKind::TriggeredAbility { .. })
            })
            .unwrap()
            .clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(original.id),
            object: None,
        }];
        g.copy_target_ability(&copier, 0);
        assert!(
            g.state
                .stack
                .iter()
                .any(|item| item.is_copy && item.controller == player.other())
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(stats(&g, attacker), (6, 6));
            assert!(g.keywords(attacker).contains(Keywords::TRAMPLE));
            g.finish_cleanup();
            assert_eq!(stats(&g, attacker), (2, 2));
            assert!(!g.keywords(attacker).contains(Keywords::TRAMPLE));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn delayed_garruk_groups_again_on_second_combat_and_bonus_survives_blockers() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = garruk_schedule(&mut g, player);
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let blocker_a = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let blocker_b = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        // A later controller of Garruk does not acquire the delayed ability.
        g.change_controller(g.object_ref(source), player.other());
        garruk_attack(&mut g, first);
        settle(&mut g);
        assert_eq!(stats(&g, first), (4, 4));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.step = Step::Blockers;
            g.state.pending = Pending::Blockers;
            act(
                &mut g,
                Action::Block {
                    blocker: blocker_a,
                    attacker: first,
                },
            );
            act(
                &mut g,
                Action::Block {
                    blocker: blocker_b,
                    attacker: first,
                },
            );
            act(&mut g, Action::FinishBlockers);
            assert_eq!(g.state.blocks.len(), 2);
            assert_eq!(stats(&g, first), (4, 4));
            assert!(g.keywords(first).contains(Keywords::TRAMPLE));
            // Simulate the next combat declaration without cleanup; the earlier
            // bonus is still active and the delayed trigger remains registered.
            g.state.attackers.clear();
            g.state.blocking.clear();
            g.state.blocks.clear();
            garruk_attack(&mut g, second);
            settle(&mut g);
            assert_eq!(stats(&g, first), (4, 4));
            assert_eq!(stats(&g, second), (5, 5));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn delayed_garruk_uses_effect_controller_and_expires_at_that_players_next_turn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = garruk_schedule(&mut g, player);
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(source), player.other());
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.active = player.other();
            g.begin_turn();
            assert_eq!(
                g.state
                    .delayed_opponent_attack_until_next_turn
                    .as_ref()
                    .unwrap()
                    .len(),
                1
            );
            g.state.active = player;
            g.begin_turn();
            assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
            garruk_attack(&mut g, first);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.card == source)
            );
            g.restore(&saved).unwrap();
        }
    }
}
