//! Independent composition and actual outside-action history regressions.
use super::*;

#[test]
fn actual_outside_actions_remain_unrepresentable_after_resolution_and_restore() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let mut pools = [vec![], vec![]];
        pools[player.index()] = vec![Card::from_name("Shock").unwrap(), Card::Forest];
        g.state.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::CasualCollection,
            pools,
            choice: None,
            known: [Vec::new(), Vec::new()],
        }));
        let id = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[id as usize].card = Card::from_name("Extrapolate the Impossible").unwrap();
        act(&mut g, Action::AnnounceCast { card: id });
        pay(&mut g);
        for _ in 0..48 {
            if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::OutsideGameChoice { .. }));
        let before = g.snapshot();
        for _ in 0..2 {
            let mut journal = Vec::new();
            for action in [
                Action::ChooseOutsideCard { index: 0 },
                Action::CancelOutsideCard,
                Action::ChooseOutsideCard { index: 0 },
                Action::ChooseOutsideCard { index: 1 },
                Action::ChooseOutsideRevealed { index: 0 },
            ] {
                act(&mut g, action.clone());
                journal.push(action);
                assert!(!prior_outside_state_supported(&g.state));
            }
            settle(&mut g);
            assert!(g.state.outside_game.is_some());
            assert!(!prior_outside_state_supported(&g.state));
            assert!(project_prior_outside(&serde_json::to_string(&journal).unwrap()).is_err());
            for viewer in [Player::P0, Player::P1] {
                assert!(
                    project_prior_outside(&serde_json::to_string(&g.observe(viewer)).unwrap())
                        .is_err()
                );
            }
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), before);
        }
    }
}

#[test]
fn outside_then_schema188_projection_preserves_old_and_schema188_only_values() {
    let old = r#"[{"card":"Bear Cub","jace_loyalty_anytime":true}]"#;
    let projected188 = project_prior_outside(old).unwrap();
    assert_eq!(projected188, old);
    assert_eq!(project_prior_schema188(&projected188).unwrap(), old);
    for payload in [
        r#"[{"card":"Seasoned Cryomancer"}]"#,
        r#"[{"nested":{"reflexive_target_limit":0}}]"#,
    ] {
        let projected188 = project_prior_outside(payload).unwrap();
        assert_eq!(projected188, payload);
        assert!(project_prior_schema188(&projected188).is_err());
    }
    let outside = r#"[{"known_outside":[[],[]]}]"#;
    assert!(project_prior_outside(outside).is_err());
    for schema in [188, 187] {
        let result = trace(Card::from_name("Bear Cub").unwrap(), Some(schema));
        assert_eq!(result.status, "completed");
        assert!(!result.observations.starts_with("UNREPRESENTABLE:"));
    }
}

#[test]
fn retained_188_card_in_hidden_checkpoint_is_accepted_by_189_only_projection() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[id as usize].card = Card::from_name("Seasoned Cryomancer").unwrap();
        g.begin_attack_rollback(player);
        g.state.objects[id as usize].card = Card::from_name("Bear Cub").unwrap();
        assert!(prior_outside_state_supported(&g.state));
        assert!(!prior_schema188_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema188_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_schema188_state_supported(&g.state));
    }
}
