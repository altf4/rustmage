//! Whole-card rules for exact placing-player observers and history-qualified grants.
use super::assurance::big_helpers::{
    cast, checked, named, next_turn as advance_turn, settle, setup,
};
use super::*;
fn next_turn(g: &mut Game) {
    advance_turn(g);
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..70 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn insect_count(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.identity() == "@token/3512fba4bfa3f3fcf0f1eb9a"
        })
        .count()
}
fn hex(g: &Game, id: CardId) -> bool {
    g.keywords(id).contains(Keywords::HEXPROOF)
}
fn put_one(g: &mut Game, p: Player, creature: CardId) {
    paid(g, p, "Take Up the Shield", &[Target::Card(creature)]);
}
fn declare(g: &mut Game, p: Player, source: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[source as usize].summoning_sick = false;
    checked(g, Action::Attack { card: source });
    checked(g, Action::FinishAttackers);
}
fn pay_attack(g: &mut Game, p: Player, source: CardId, pay: bool, target: CardId) {
    declare(g, p, source);
    until(g, |g| {
        g.legal_actions().contains(&Action::DeclineResolutionMana)
    });
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(target)
    }));
    if pay {
        reject(g, Action::PayResolutionMana { payment: [0; 6] });
        checked(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        until(g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        });
        let land = g.state.objects.iter().enumerate().find_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.card.is_land()).then_some(i as CardId)
        });
        if let Some(land) = land {
            reject(
                g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
        }
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(g, Action::FinishTargets);
        }
    } else {
        checked(g, Action::DeclineResolutionMana);
    }
    settle(g);
}
#[test]
fn placement_kid_paid_second_draw_private_first_third_and_opponent_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Opt", &[]); // First draw predates Kid Loki's entry.
        let kid = paid(&mut g, p, "Kid Loki", &[]);
        assert!(!hex(&g, kid));
        let top = *g.state.players[p.index()].library.last().unwrap();
        paid(&mut g, p, "Opt", &[]);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 1);
        assert!(hex(&g, kid));
        assert!(g.observe(p).objects.iter().any(|o| o.reference.card == top));
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == top)
        );
        paid(&mut g, p, "Opt", &[]);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 1);
        next_turn(&mut g);
        assert!(!hex(&g, kid));
        paid(&mut g, p.other(), "Opt", &[]);
        paid(&mut g, p.other(), "Opt", &[]);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 1);
        paid(&mut g, p, "Opt", &[]);
        paid(&mut g, p, "Opt", &[]);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 2);
        assert!(hex(&g, kid));
    }
}
#[test]
fn placement_kid_history_predates_entry_and_removal_preserves_hexproof_until_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        put_one(&mut g, p, bear);
        let kid = paid(&mut g, p, "Kid Loki", &[]);
        assert!(hex(&g, bear));
        assert!(!hex(&g, kid));
        g.pay_counters(bear, &Counters::of(CounterKind::PlusOne, 1));
        assert!(hex(&g, bear));
        // Opposing targeted removal cannot target the history-qualified creature.
        checked(&mut g, Action::Pass);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: shock });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        next_turn(&mut g);
        assert!(!hex(&g, bear));
    }
}
#[test]
fn placement_kid_exact_placer_kind_controller_and_incarnation_filters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kid = paid(&mut g, p, "Kid Loki", &[]);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        g.effect_counter_by(
            own,
            CounterKind::PlusOne,
            1,
            p.other(),
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            kid,
            CounterKind::Stun,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            enemy,
            CounterKind::PlusOne,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            artifact,
            CounterKind::PlusOne,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert!(!hex(&g, own));
        assert!(!hex(&g, enemy));
        assert!(!hex(&g, artifact));
        assert!(!hex(&g, kid));
        put_one(&mut g, p, own);
        assert!(hex(&g, own));
        let reference = g.object_ref(own);
        g.change_controller(reference, p.other());
        assert!(!hex(&g, own));
        g.change_controller(reference, p);
        assert!(hex(&g, own));
        let source = g.object_ref(kid);
        g.change_controller(source, p.other());
        assert!(!hex(&g, enemy));
        put_one(&mut g, p.other(), enemy);
        assert!(hex(&g, enemy));
        assert!(!hex(&g, own));
        g.change_controller(source, p);
        assert!(hex(&g, own));
        g.move_object(own, Zone::Exile);
        g.move_object(own, Zone::Battlefield);
        assert_ne!(g.object_ref(own), reference);
        assert!(!hex(&g, own));
    }
}
#[test]
fn placement_kid_source_departure_blink_and_ability_suppression_are_continuous() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kid = paid(&mut g, p, "Kid Loki", &[]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        put_one(&mut g, p, bear);
        assert!(hex(&g, bear));
        let moon = paid(&mut g, p, "Imprisoned in the Moon", &[Target::Card(kid)]);
        assert!(!hex(&g, bear));
        g.move_object(moon, Zone::Graveyard);
        assert!(hex(&g, bear));
        g.move_object(kid, Zone::Exile);
        assert!(!hex(&g, bear));
        g.move_object(kid, Zone::Battlefield);
        assert!(hex(&g, bear));
        assert!(!hex(&g, kid));
        let moon = paid(&mut g, p, "Imprisoned in the Moon", &[Target::Card(bear)]);
        assert!(!hex(&g, bear));
        g.move_object(moon, Zone::Graveyard);
        assert!(hex(&g, bear));
    }
}
#[test]
fn placement_kid_queued_second_draw_old_object_and_captured_counter_placer() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let kid = paid(&mut g, p, "Kid Loki", &[]);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            until(&mut g, |g| {
                g.state.stack.iter().any(|item| {
                    item.object.card == kid
                        && matches!(item.kind, StackKind::TriggeredAbility { .. })
                })
            });
            let old = g.object_ref(kid);
            if blink {
                g.move_object(kid, Zone::Exile);
                g.move_object(kid, Zone::Battlefield);
            } else {
                g.change_controller(old, p.other());
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(kid, CounterKind::PlusOne),
                u64::from(!blink)
            );
            assert!(!hex(&g, kid));
            if !blink {
                g.change_controller(old, p);
                assert!(hex(&g, kid));
            }
        }
    }
}
#[test]
fn placement_ant_paid_attack_optional_payment_separate_targets_self_own_and_opponent() {
    for p in [Player::P0, Player::P1] {
        for target_case in 0..3 {
            let mut g = setup(p);
            let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Forest", Zone::Battlefield);
            let target = [ant, own, enemy][target_case];
            pay_attack(&mut g, p, ant, true, target);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
            assert_eq!(insect_count(&g, p), 1);
            assert_eq!(insect_count(&g, p.other()), 0);
        }
        let mut g = setup(p);
        let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
        pay_attack(&mut g, p, ant, false, ant);
        assert_eq!(g.counter_count(ant, CounterKind::PlusOne), 0);
        assert_eq!(insect_count(&g, p), 0);
    }
}
#[test]
fn placement_ant_any_creature_counter_not_only_attack_and_once_survives_control() {
    for p in [Player::P0, Player::P1] {
        for target_case in 0..3 {
            let mut g = setup(p);
            let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            put_one(&mut g, p, [ant, own, enemy][target_case]);
            assert_eq!(insect_count(&g, p), 1);
            put_one(&mut g, p, own);
            assert_eq!(insect_count(&g, p), 1);
            let reference = g.object_ref(ant);
            g.change_controller(reference, p.other());
            put_one(&mut g, p.other(), ant);
            assert_eq!(insect_count(&g, p.other()), 0);
            next_turn(&mut g);
            put_one(&mut g, p.other(), ant);
            assert_eq!(insect_count(&g, p.other()), 1);
        }
    }
}
#[test]
fn placement_ant_rejects_wrong_placer_kind_zero_and_noncreature_without_consuming_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        g.effect_counter_by(
            ant,
            CounterKind::PlusOne,
            1,
            p.other(),
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            ant,
            CounterKind::Stun,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            artifact,
            CounterKind::PlusOne,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.effect_counter_by(
            enemy,
            CounterKind::PlusOne,
            0,
            p,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        assert_eq!(insect_count(&g, p), 0);
        put_one(&mut g, p, enemy);
        assert_eq!(insect_count(&g, p), 1);
    }
}
#[test]
fn placement_ant_multi_counter_and_token_replacements_and_global_counter_event_only_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
        paid(&mut g, p, "Political Triumph", &[]);
        for _ in 0..2 {
            paid(&mut g, p, "Bear Cub", &[]);
        }
        assert_eq!(g.counter_count(ant, CounterKind::PlusOne), 2);
        assert_eq!(insect_count(&g, p), 2);
        for (id, o) in g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        {
            assert_eq!(o.card.identity(), "@token/3512fba4bfa3f3fcf0f1eb9a");
            assert_eq!(o.card.colors(), 16);
            assert_eq!(
                g.stats(id as CardId),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert_eq!(o.card.keywords(), Keywords::NONE);
        }
    }
}
#[test]
fn placement_ant_queued_token_keeps_controller_after_departure_and_blink_resets_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Take Up the Shield", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(bear)]);
        until(&mut g, |g| {
            g.state.stack.iter().any(|item| {
                item.object.card == ant && matches!(item.kind, StackKind::TriggeredAbility { .. })
            })
        });
        let old = g.object_ref(ant);
        g.move_object(ant, Zone::Exile);
        settle(&mut g);
        assert_eq!(insect_count(&g, p), 1);
        g.move_object(ant, Zone::Battlefield);
        assert_ne!(g.object_ref(ant), old);
        put_one(&mut g, p, ant);
        assert_eq!(insect_count(&g, p), 2);
    }
}
#[test]
fn placement_ant_reflexive_target_blink_fizzles_and_departed_attack_still_places_counter() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = setup(p);
            let ant = paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            declare(&mut g, p, ant);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            if source_leaves {
                g.move_object(ant, Zone::Exile);
            }
            checked(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(bear),
                })
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            if !source_leaves {
                g.move_object(bear, Zone::Exile);
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(bear, CounterKind::PlusOne),
                u64::from(source_leaves)
            );
            assert_eq!(insect_count(&g, p), 0);
        }
    }
}

#[test]
fn placement_entry_counters_qualify_history_and_trigger_the_observer_only_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Kid Loki", &[]);
        paid(&mut g, p, "Ant-Man, Colony Commander", &[]);
        let hydra = paid(&mut g, p, "Mossborn Hydra", &[]);
        assert_eq!(g.counter_count(hydra, CounterKind::PlusOne), 1);
        assert!(hex(&g, hydra));
        assert_eq!(insect_count(&g, p), 1);
        let bane = paid(&mut g, p, "Heroes' Bane", &[]);
        assert_eq!(g.counter_count(bane, CounterKind::PlusOne), 4);
        assert!(hex(&g, bane));
        assert_eq!(insect_count(&g, p), 1);
    }
}
