//! Independent target, copy and replacement regressions for Seasoned Cryomancer.
use super::*;

fn setup_reflexive(g: &mut Game, p: Player, cards: [&str; 2]) -> (CardId, [CardId; 2]) {
    let source = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[source as usize].card = Card::from_name("Seasoned Cryomancer").unwrap();
    let hand = [
        put(g, p, cards[0], Zone::Hand),
        put(g, p, cards[1], Zone::Hand),
    ];
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    for _ in 0..64 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            break;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
    (source, hand)
}
fn discard_pair(g: &mut Game, hand: [CardId; 2]) {
    for card in hand {
        act(g, Action::Discard { card });
    }
}
fn target_choices(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::FinishTargets))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("reflexive target choice not reached: {:?}", g.state.pending);
}

#[test]
fn seasoned_reflexive_allows_zero_and_rejects_more_than_discard_count() {
    for p in [Player::P0, Player::P1] {
        for zero in [false, true] {
            let mut g = scenario(p);
            let one = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let two = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let (_, hand) = setup_reflexive(&mut g, p, ["Shock", "Forest"]);
            discard_pair(&mut g, hand);
            target_choices(&mut g);
            let before = g.snapshot();
            for _ in 0..2 {
                if !zero {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(one),
                        },
                    );
                }
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                let packet = g.state.stack.last().unwrap();
                assert_eq!(packet.reflexive_target_limit, Some(1));
                assert_eq!(packet.targets.len(), usize::from(!zero));
                let rejected = g.snapshot();
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(two),
                    },
                );
                assert_eq!(g.snapshot(), rejected);
                for viewer in [p, p.other()] {
                    let json = serde_json::to_value(g.observe(viewer)).unwrap();
                    let view = json["stack"].as_array().unwrap().last().unwrap();
                    assert_eq!(view["reflexive_target_limit"], 1);
                }
                settle(&mut g);
                assert_eq!(g.counter_count(one, CounterKind::Stun), u64::from(!zero));
                assert_eq!(g.counter_count(two, CounterKind::Stun), 0);
                g.restore(&before).unwrap();
            }
        }
    }
}

#[test]
fn seasoned_two_targets_partial_invalidation_preserves_other_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let one = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let two = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let (_, hand) = setup_reflexive(&mut g, p, ["Shock", "Opt"]);
        discard_pair(&mut g, hand);
        target_choices(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(one),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(two),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let before = g.snapshot();
        for _ in 0..2 {
            g.move_object(one, Zone::Exile);
            g.move_object(one, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.counter_count(one, CounterKind::Stun), 0);
            assert_eq!(g.counter_count(two, CounterKind::Stun), 1);
            assert!(g.object_view(two).tapped);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn seasoned_competing_discard_replacements_suspend_and_replay() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let (_, hand) = setup_reflexive(&mut g, p, ["Progenitus", "Forest"]);
            act(&mut g, Action::Discard { card: hand[0] });
            act(&mut g, Action::Discard { card: hand[1] });
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { player } if player == p));
            let before = g.snapshot();
            for _ in 0..2 {
                let effect = if exile {
                    ZoneReplacementKind::Exile
                } else {
                    ZoneReplacementKind::RevealShuffle
                };
                act(&mut g, Action::ChooseZoneReplacement { effect });
                assert_eq!(
                    g.object_view(hand[0]).zone,
                    if exile { Zone::Exile } else { Zone::Library }
                );
                target_choices(&mut g);
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                settle(&mut g);
                assert_eq!(g.counter_count(target, CounterKind::Stun), 1);
                g.restore(&before).unwrap();
            }
        }
    }
}

#[test]
fn seasoned_copied_reflexive_retargets_with_same_public_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let original_target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let copied_target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let (_, hand) = setup_reflexive(&mut g, p, ["Shock", "Forest"]);
        discard_pair(&mut g, hand);
        target_choices(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(original_target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(original.id),
            object: None,
        }];
        g.state.resolving = Some(Resolution {
            chosen_creature_type: None,
            target_counter_removal: None,
            linked_copy: None,
            cascade: None,
            target_card_facts: None,
            immediate_blink_return: None,
            connive: None,
            explore: None,
            evidence: None,
            item: copier.clone(),
            mana_payment: None,
            forage: None,
            target_library: None,
            program: &[],
            pc: 0,
            target_group: 0,
            moved_count: None,
            remembered_draws: None,
            remembered_creatures: Vec::new(),
            sacrificed_power: None,
            discard_after: None,
            optional_wheel: None,
            proliferate: None,
            discards: [0; 2],
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            milled_selection: None,
            library: None,
            selection: None,
            return_attachment: None,
            search: None,
            search_result: None,
            optional_searches: None,
            choice: None,
        });
        assert!(g.copy_target_ability(&copier, 0));
        let before = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(copied_target),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            let copied = g.state.stack.last().unwrap();
            assert!(copied.is_copy);
            assert_eq!(copied.reflexive_target_limit, Some(1));
            for viewer in [p, p.other()] {
                let json = serde_json::to_value(g.observe(viewer)).unwrap();
                assert_eq!(
                    json["stack"].as_array().unwrap().last().unwrap()["reflexive_target_limit"],
                    1
                );
            }
            settle(&mut g);
            assert_eq!(g.counter_count(original_target, CounterKind::Stun), 1);
            assert_eq!(g.counter_count(copied_target, CounterKind::Stun), 1);
            g.restore(&before).unwrap();
        }
    }
}
