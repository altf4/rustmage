//! Artifact sacrifice and controller-sensitive departure interactions.
use super::*;
use crate::engine::untap_rules::UntapCause;

#[test]
fn relic_entry_departure_optional_stun_and_opponent_sacrifice_damage() {
    for p in [Player::P0, Player::P1] {
        for target in [false, true] {
            for tracker_enemy in [false, true] {
                for replaced in [false, true] {
                    let mut g = scenario(p);
                    let tracker_owner = if tracker_enemy { p.other() } else { p };
                    let tracker = put(&mut g, tracker_owner, "Vengeful Tracker", Zone::Battlefield);
                    assert_eq!(stats(&g, tracker), (2, 2));
                    if replaced {
                        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
                    }
                    let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let untapped = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    g.state.objects[victim as usize].tapped = true;
                    let relic = put(&mut g, p, "Cryogen Relic", Zone::Hand);
                    let life = g.state.players[p.index()].life;
                    let initial_draws = g.state.players[p.index()].cards_drawn_this_turn;
                    big_helpers::cast(&mut g, relic, None, &[]);
                    big_helpers::settle(&mut g);
                    assert_eq!(
                        g.state.players[p.index()].cards_drawn_this_turn,
                        initial_draws + 1
                    );
                    big_helpers::checked(
                        &mut g,
                        Action::AnnounceActivate {
                            card: relic,
                            ability: 0,
                            counter_payment: Counters::default(),
                        },
                    );
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(untapped),
                        },
                    );
                    if target {
                        big_helpers::checked(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(victim),
                            },
                        );
                    } else {
                        big_helpers::checked(&mut g, Action::FinishTargets);
                    }
                    pay(&mut g);
                    big_helpers::settle(&mut g);
                    assert_eq!(
                        g.object_view(relic).zone,
                        if replaced {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                    assert_eq!(
                        g.state.players[p.index()].cards_drawn_this_turn,
                        initial_draws + 2
                    );
                    assert_eq!(
                        g.counter_count(victim, CounterKind::Stun),
                        u64::from(target)
                    );
                    assert_eq!(
                        g.state.players[p.index()].life,
                        life - if tracker_enemy { 2 } else { 0 }
                    );
                }
            }
        }
    }
}

#[test]
fn relic_leaves_without_dying_and_graveyard_changes_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let relic = put(&mut g, p, "Cryogen Relic", Zone::Battlefield);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        g.move_object(relic, Zone::Hand);
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        g.move_object(relic, Zone::Graveyard);
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
    }
}

#[test]
fn relic_own_target_untap_replacement_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for loses_tapped in [false, true] {
            let mut g = scenario(p);
            let relic = put(&mut g, p, "Cryogen Relic", Zone::Battlefield);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[victim as usize].tapped = true;
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            big_helpers::checked(
                &mut g,
                Action::AnnounceActivate {
                    card: relic,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            pay(&mut g);
            if loses_tapped {
                g.untap_objects(&[victim], UntapCause::Effect);
            }
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
            assert_eq!(
                g.counter_count(victim, CounterKind::Stun),
                u64::from(!loses_tapped)
            );
            if !loses_tapped {
                g.untap_objects(&[victim], UntapCause::Effect);
                assert!(g.object_view(victim).tapped);
                assert_eq!(g.counter_count(victim, CounterKind::Stun), 0);
                g.untap_objects(&[victim], UntapCause::Effect);
                assert!(!g.object_view(victim).tapped);
            }
        }
    }
}

#[test]
fn tracker_ignores_opponent_sacrificing_nonartifact() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Vengeful Tracker", Zone::Battlefield);
        let land = put(&mut g, p, "Evolving Wilds", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, life);
    }
}

#[test]
fn tracker_uses_frozen_sacrifice_and_controller_after_source_departure_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for tracker_departs in [false, true] {
            let mut g = scenario(p);
            // The sacrificer controls an artifact owned by the Tracker's controller.
            let tracker = put(&mut g, p.other(), "Vengeful Tracker", Zone::Battlefield);
            let relic = put(&mut g, p.other(), "Cryogen Relic", Zone::Battlefield);
            g.state.objects[relic as usize].controller = p;
            let life = g.state.players[p.index()].life;
            let other_life = g.state.players[p.other().index()].life;
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            big_helpers::checked(
                &mut g,
                Action::AnnounceActivate {
                    card: relic,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            // Both creatures are untapped, so the optional target group completes automatically.
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseTarget { .. }))
            );
            pay(&mut g);
            if tracker_departs {
                g.move_object(tracker, Zone::Hand);
            } else {
                g.state.objects[tracker as usize].controller = p;
            }
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life - 2);
            assert_eq!(g.state.players[p.other().index()].life, other_life);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
            assert_eq!(g.object_view(relic).zone, Zone::Graveyard);
            assert_eq!(g.object_view(relic).owner, p.other());
        }
    }
}

#[test]
fn tracker_sees_each_opponent_artifact_in_simultaneous_departure_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tracker = put(&mut g, p.other(), "Vengeful Tracker", Zone::Battlefield);
        let first = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let second = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let before = g.snapshot();
        for _ in 0..2 {
            g.sacrifice_batch(&[(p.other(), tracker), (p, first), (p, second)]);
            assert_eq!(g.object_view(tracker).zone, Zone::Graveyard);
            assert_eq!(g.object_view(first).zone, Zone::Graveyard);
            assert_eq!(g.object_view(second).zone, Zone::Graveyard);
            big_helpers::settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life - 4);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn tracker_sees_mana_ability_sacrifice_but_not_prohibited_sacrifice() {
    use crate::engine::ability_world::TEST_SACRIFICE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Vengeful Tracker", Zone::Battlefield);
        let treasure = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let before_mana = g.state.players[p.index()].mana;
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: [0, 1, 0, 0, 0, 0], .. } if *card == treasure)).expect("blue Treasure mana action");
        TEST_SACRIFICE_PROHIBITOR.with(|slot| slot.set(Some(g.object_ref(provider))));
        reject(&mut g, action.clone());
        g.sacrifice_batch(&[(p, treasure)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(treasure).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life);
        assert_eq!(g.state.players[p.index()].mana, before_mana);
        TEST_SACRIFICE_PROHIBITOR.with(|slot| slot.set(None));
        big_helpers::checked(&mut g, action);
        assert_ne!(g.object_view(treasure).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana[1], before_mana[1] + 1);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 2);
    }
}
