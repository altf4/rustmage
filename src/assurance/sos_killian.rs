//! Combat batches, optional hybrid mana, and exact graveyard incarnation returns.
use super::big_helpers::{cast as paid, checked, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..30 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("choice expected");
}
#[test]
fn sos_killian_paid_boost_draw_and_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let body = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Killian's Confidence", Zone::Hand);
            let hand = g.hand_count(p) - 1;
            paid(&mut g, spell, None, &[Target::Card(body)]);
            if fizzle {
                g.move_object(body, Zone::Hand);
                g.move_object(body, Zone::Battlefield);
            }
            finish(&mut g);
            assert_eq!(stats(&g, body), if fizzle { (2, 2) } else { (3, 3) });
            assert_eq!(g.hand_count(p), hand + usize::from(!fizzle));
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(g.state.damage_sources.contains(&spell));
        }
    }
}
#[test]
fn sos_killian_actual_simultaneous_combat_uses_one_optional_hybrid_payment_window() {
    for p in [Player::P0, Player::P1] {
        for route in 0..4 {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Killian's Confidence", Zone::Graveyard);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let land = put(
                &mut g,
                p,
                if route == 1 { "Swamp" } else { "Plains" },
                Zone::Battlefield,
            );
            g.state.players[p.index()].mana = [0; 6];
            if route == 3 {
                g.state.objects[land as usize].tapped = true;
            }
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: a });
            checked(&mut g, Action::Attack { card: b });
            checked(&mut g, Action::FinishAttackers);
            let mut windows = 0;
            for _ in 0..100 {
                if g.state.step == Step::Main2 {
                    break;
                }
                if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                    windows += 1;
                    let request = g.observe(p).resolution_payment.unwrap();
                    assert_eq!(request.cost.colored, [1, 0, 0, 0, 0, 0]);
                    assert_eq!(request.alternatives.len(), 1);
                    reject(
                        &mut g,
                        Action::PayResolutionMana {
                            payment: [0, 1, 0, 0, 0, 0],
                        },
                    );
                    if route < 2 {
                        let tap = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::TapMana { card } if *card == land))
                            .unwrap();
                        checked(&mut g, tap);
                        let payment = if route == 1 {
                            [0, 0, 1, 0, 0, 0]
                        } else {
                            [1, 0, 0, 0, 0, 0]
                        };
                        checked(&mut g, Action::PayResolutionMana { payment });
                    } else {
                        checked(&mut g, Action::DeclineResolutionMana);
                    }
                } else if g.state.pending == Pending::Blockers {
                    checked(&mut g, Action::FinishBlockers);
                } else {
                    checked(&mut g, Action::Pass);
                }
            }
            assert_eq!(windows, 1);
            assert_eq!(g.state.players[p.other().index()].life, 16);
            assert_eq!(
                g.object_view(spell).zone,
                if route < 2 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn sos_killian_graveyard_index_and_queued_return_are_exact_and_combat_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Killian's Confidence", Zone::Graveyard);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(spell, Zone::Hand);
        g.move_object(spell, Zone::Graveyard);
        assert_eq!(
            g.state
                .damage_sources
                .iter()
                .filter(|&&id| id == spell)
                .count(),
            1
        );
        let event = |g: &Game, source, combat| DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target: Target::Player(p.other()),
            amount: 1,
            combat,
        };
        g.apply_damage_batch(&[event(&g, own, false), event(&g, other, true)]);
        assert!(g.state.pending_triggers.is_empty());
        g.apply_damage_batch(&[event(&g, own, true)]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let old = g.object_ref(spell);
        g.move_object(spell, Zone::Exile);
        g.move_object(spell, Zone::Graveyard);
        assert_ne!(g.object_ref(spell), old);
        pause(&mut g);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        g.move_object(spell, Zone::Hand);
        g.apply_damage_batch(&[event(&g, own, true)]);
        assert!(g.state.pending_triggers.is_empty());
    }
}
