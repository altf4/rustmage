//! Gated Seasoned Cryomancer, actual discard count and graveyard activation.
use super::*;

fn cryomancer(g: &mut Game, p: Player, zone: Zone) -> CardId {
    let card = put(g, p, "Bear Cub", zone);
    g.state.objects[card as usize].card = Card::from_name("Seasoned Cryomancer").unwrap();
    if zone == Zone::Graveyard {
        g.register_graveyard_sources(card);
    }
    card
}

#[test]
fn seasoned_discards_create_one_separate_trigger_with_actual_target_limit() {
    for player in [Player::P0, Player::P1] {
        for nonlands in 0..=2 {
            let mut g = scenario(player);
            let source = cryomancer(&mut g, player, Zone::Hand);
            let one = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let two = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let hand: Vec<_> = (0..2)
                .map(|i| {
                    put(
                        &mut g,
                        player,
                        if i < nonlands { "Shock" } else { "Forest" },
                        Zone::Hand,
                    )
                })
                .collect();
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            for _ in 0..40 {
                g.prepare_triggers();
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            let saved = g.snapshot();
            for _ in 0..2 {
                for &card in &hand {
                    act(&mut g, Action::Discard { card });
                }
                for _ in 0..40 {
                    g.prepare_triggers();
                    if g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(one),
                    }) {
                        break;
                    }
                    if nonlands == 0
                        && g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                    {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
                if nonlands > 0 {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(one),
                        },
                    );
                    if nonlands == 2 {
                        act(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(two),
                            },
                        );
                    }
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    }
                    settle(&mut g);
                    assert!(g.object_view(one).tapped);
                    assert_eq!(g.counter_count(one, CounterKind::Stun), 1);
                    assert_eq!(
                        g.counter_count(two, CounterKind::Stun),
                        u64::from(nonlands == 2)
                    );
                } else {
                    assert_eq!(g.counter_count(one, CounterKind::Stun), 0);
                }
                g.restore(&saved).unwrap();
                assert_eq!(g.snapshot(), saved);
            }
        }
    }
}

#[test]
fn seasoned_exiles_itself_as_cost_and_draws_two_at_instant_speed() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = cryomancer(&mut g, player, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player;
        g.state.players[player.index()].mana = [0, 2, 0, 0, 0, 3];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::AnnounceActivate {
                    card,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}

fn cryomancer_until_discard(g: &mut Game, source: CardId) {
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    for _ in 0..64 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("discard choice not reached: {:?}", g.state.pending);
}

#[test]
fn seasoned_replaced_discards_count_and_trigger_survives_source_departure() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let source = cryomancer(&mut g, player, Zone::Hand);
        let discarded = put(&mut g, player, "Shock", Zone::Hand);
        let land = put(&mut g, player, "Forest", Zone::Hand);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cryomancer_until_discard(&mut g, source);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::Discard { card: discarded });
            act(&mut g, Action::Discard { card: land });
            assert_eq!(g.object_view(discarded).zone, Zone::Exile);
            g.move_object(source, Zone::Exile);
            for _ in 0..48 {
                g.prepare_triggers();
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(victim),
                }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert!(g.object_view(victim).tapped);
            assert_eq!(g.counter_count(victim, CounterKind::Stun), 1);
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}

#[test]
fn seasoned_miracle_interrupts_draw_before_discard_and_snapshot_replays() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        // Group Project has no intrinsic miracle; Lorehold grants the
        // first-draw reveal and miracle ability to instant/sorcery cards.
        put(&mut g, player, "Lorehold, the Historian", Zone::Battlefield);
        let source = cryomancer(&mut g, player, Zone::Hand);
        let miracle = put(&mut g, player, "Group Project", Zone::Library);
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..64 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(g.state.pending, Pending::MiracleReveal { player: p } if p == player));
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            g.observe(player).miracle.as_ref().unwrap().object.card,
            miracle
        );
        assert!(g.observe(player.other()).miracle.is_none());
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::DeclineMiracle);
            assert!(
                matches!(g.state.pending, Pending::EffectDiscard { player: p, .. } if p == player)
            );
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            assert_eq!(g.object_view(miracle).zone, Zone::Hand);
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .all(|item| item.source.expect("object-backed packet").id != source)
            );
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}

#[test]
fn seasoned_target_leaving_after_trigger_is_announced_cannot_receive_stun() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = cryomancer(&mut g, player, Zone::Hand);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        let land = put(&mut g, player, "Forest", Zone::Hand);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cryomancer_until_discard(&mut g, source);
        act(&mut g, Action::Discard { card: shock });
        act(&mut g, Action::Discard { card: land });
        for _ in 0..48 {
            g.prepare_triggers();
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim),
            }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(victim, Zone::Exile);
            g.move_object(victim, Zone::Battlefield);
            settle(&mut g);
            assert!(!g.object_view(victim).tapped);
            assert_eq!(g.counter_count(victim, CounterKind::Stun), 0);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn seasoned_short_library_discards_the_only_card_actually_drawn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = cryomancer(&mut g, player, Zone::Hand);
        g.state.players[player.index()].library.clear();
        let only = put(&mut g, player, "Forest", Zone::Library);
        cryomancer_until_discard(&mut g, source);
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
        assert!(g.state.players[player.index()].attempted_empty_draw);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::Discard { card: only });
            assert_eq!(g.object_view(only).zone, Zone::Graveyard);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|identity| identity.id == source))
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn seasoned_copied_reflexive_preserves_target_limit_and_resolves_twice() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = cryomancer(&mut g, player, Zone::Hand);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        let land = put(&mut g, player, "Forest", Zone::Hand);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cryomancer_until_discard(&mut g, source);
        act(&mut g, Action::Discard { card: shock });
        act(&mut g, Action::Discard { card: land });
        for _ in 0..48 {
            g.prepare_triggers();
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim),
            }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.reflexive_target_limit, Some(1));
        assert_eq!(original.x, None);
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
        act(&mut g, Action::FinishStackTargets);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.counter_count(victim, CounterKind::Stun), 2);
            g.restore(&saved).unwrap();
        }
    }
}
