//! Whole Sentry/Alien rules using actual paid casts and owned checked decisions.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
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
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
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
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn tokens(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.name() == name)
                .then_some(id as CardId)
        })
        .collect()
}
fn begin_combat(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.begin_step(Step::BeginCombat);
    g.prepare_triggers();
}
fn finish(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished announcement");
}
fn sentry_to_target(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "The Sentry, Golden Guardian", Zone::Hand);
    cast(g, id, None, &[]);
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(p.other()),
        })
    });
    id
}

#[test]
fn msh_sentry_alien_sentry_paid_stats_keywords_opponent_target_and_exact_void_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        let source = sentry_to_target(&mut g, p);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            (g.object_view(source).power, g.object_view(source).toughness),
            (5, 5)
        );
        assert!(
            g.keywords(source).contains(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        let void = tokens(&g, p.other(), "The Void");
        assert_eq!(void.len(), 1);
        assert!(tokens(&g, p, "The Void").is_empty());
        let id = void[0];
        let v = g.object_view(id);
        assert_eq!((v.power, v.toughness, v.kinds, v.colors), (5, 5, 1, 4));
        assert!(g.has_subtype(id, "Horror") && g.has_subtype(id, "Villain"));
        assert!(g.state.objects[id as usize].card.is_legendary());
        assert!(
            g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::INDESTRUCTIBLE))
        );
        assert!(g.attacks_each_combat(id));
    }
}

#[test]
fn msh_sentry_alien_sentry_queued_trigger_preserves_controller_and_target_after_source_changes() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let source = sentry_to_target(&mut g, p);
            if depart {
                g.move_object(source, Zone::Exile);
            } else {
                let r = g.object_ref(source);
                g.change_controller(r, p.other());
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(tokens(&g, p.other(), "The Void").len(), 1);
            assert!(tokens(&g, p, "The Void").is_empty());
        }
    }
}

#[test]
fn msh_sentry_alien_sentry_target_player_hexproof_before_and_after_binding_prevents_void() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        paid(&mut g, p, "The Sentry, Golden Guardian", &[]);
        assert!(tokens(&g, p.other(), "The Void").is_empty());
        let mut g = setup(p);
        sentry_to_target(&mut g, p);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        settle(&mut g);
        assert!(tokens(&g, p.other(), "The Void").is_empty());
    }
}

#[test]
fn msh_sentry_alien_void_creation_uses_opponent_replacements_and_owner_legend_choice() {
    for p in [Player::P0, Player::P1] {
        for doubler_owner in [p, p.other()] {
            let mut g = setup(p);
            named(&mut g, doubler_owner, "Doubling Season", Zone::Battlefield);
            sentry_to_target(&mut g, p);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            finish(&mut g);
            if doubler_owner == p {
                settle(&mut g);
                assert_eq!(tokens(&g, p.other(), "The Void").len(), 1);
            } else {
                until(&mut g, |g| {
                    g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::KeepLegend { .. }))
                });
                assert_eq!(g.actor(), Some(p.other()));
                let made = tokens(&g, p.other(), "The Void");
                assert_eq!(made.len(), 2);
                let keep = g.object_ref(made[1]);
                checked(&mut g, Action::KeepLegend { object: keep });
                settle(&mut g);
                assert_eq!(tokens(&g, p.other(), "The Void"), vec![made[1]]);
            }
        }
    }
}

#[test]
fn msh_sentry_alien_sentry_vigilance_and_flying_combat_keep_both_indestructible_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "The Sentry, Golden Guardian", &[]);
        let void = tokens(&g, p.other(), "The Void")[0];
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(source).tapped);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: source,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: void,
                attacker: source,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        for id in [source, void] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).damage, 5);
        }
    }
}

#[test]
fn msh_sentry_alien_void_required_attacks_sickness_tap_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "The Sentry, Golden Guardian", &[]);
        let void = tokens(&g, p.other(), "The Void")[0];
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: void });
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        g.state.objects[void as usize].summoning_sick = false;
        reject(&mut g, Action::FinishAttackers);
        g.state.objects[void as usize].tapped = true;
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
        g.state.objects[void as usize].tapped = false;
        checked(&mut g, Action::Attack { card: void });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(void).tapped);
        let mut g = setup(p);
        paid(&mut g, p, "The Sentry, Golden Guardian", &[]);
        let void = tokens(&g, p.other(), "The Void")[0];
        paid(&mut g, p, "Broken Wings", &[Target::Card(void)]);
        assert_eq!(g.object_view(void).zone, Zone::Battlefield);
        paid(&mut g, p, "Witness Protection", &[Target::Card(void)]);
        assert!(!g.attacks_each_combat(void));
        assert!(!g.keywords(void).contains(Keywords::INDESTRUCTIBLE));
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[void as usize].summoning_sick = false;
        assert!(g.legal_actions().contains(&Action::FinishAttackers));
    }
}

#[test]
fn msh_sentry_alien_invasion_paid_beginning_combat_sequence_and_exact_alien_haste_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 1, 1];
        let source = paid(&mut g, p, "Alien Invasion", &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.counter_count(source, CounterKind::Invasion), 0);
        assert!(tokens(&g, p, "Alien Token").is_empty());
        begin_combat(&mut g, p.other());
        settle(&mut g);
        assert!(tokens(&g, p, "Alien Token").is_empty());
        begin_combat(&mut g, p);
        settle(&mut g);
        let first = tokens(&g, p, "Alien Token");
        assert_eq!(first.len(), 1);
        let first = first[0];
        let v = g.object_view(first);
        assert_eq!((v.power, v.toughness, v.kinds, v.colors), (1, 1, 1, 8));
        assert!(g.has_subtype(first, "Alien"));
        assert!(g.keywords(first).contains(Keywords::HASTE));
        assert!(g.attacks_each_combat(first));
        assert_eq!(g.counter_count(source, CounterKind::Invasion), 1);
        begin_combat(&mut g, p);
        settle(&mut g);
        let made = tokens(&g, p, "Alien Token");
        assert_eq!(made.len(), 2);
        assert_eq!(g.counter_count(source, CounterKind::Invasion), 2);
        let second = *made.iter().find(|&&id| id != first).unwrap();
        assert_eq!(g.counter_count(second, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(first, CounterKind::PlusOne), 0);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Attack { card: first });
        reject(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Attack { card: second });
        checked(&mut g, Action::FinishAttackers);
    }
}

#[test]
fn msh_sentry_alien_invasion_live_counter_quantity_after_queue_and_distinct_replacement_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Alien Invasion", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        g.set_counters(source, Counters::of(CounterKind::Invasion, 1));
        begin_combat(&mut g, p);
        g.set_counters(source, Counters::of(CounterKind::Invasion, 3));
        let mut token_decisions = 0;
        let mut counter_decisions = 0;
        let mut saw_tokens_without_counters = false;
        for _ in 0..30 {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
                    || (g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.resolving.is_none()
                        && g.state.pending == Pending::Priority)
            });
            if !matches!(g.state.pending, Pending::Replacement { .. }) {
                break;
            }
            let choice = g.observe(g.actor().unwrap()).replacement_choice.unwrap();
            if choice.kind == "Tokens" {
                token_decisions += 1;
                assert!(tokens(&g, p, "Alien Token").is_empty());
            } else {
                counter_decisions += 1;
                let made = tokens(&g, p, "Alien Token");
                if !made.is_empty()
                    && made
                        .iter()
                        .all(|&id| g.counter_count(id, CounterKind::PlusOne) == 0)
                {
                    saw_tokens_without_counters = true;
                }
                assert_eq!(g.counter_count(source, CounterKind::Invasion), 3);
            }
            let e = choice.effects[0];
            checked(
                &mut g,
                Action::ChooseReplacement {
                    source: e.source,
                    ability: e.ability,
                },
            );
        }
        settle(&mut g);
        let made = tokens(&g, p, "Alien Token");
        assert_eq!(made.len(), 4);
        assert_eq!(token_decisions, 1);
        assert_eq!(counter_decisions, 5);
        assert!(saw_tokens_without_counters);
        for id in made {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 12);
            assert_eq!(g.object_view(id).power, 13);
        }
        assert_eq!(g.counter_count(source, CounterKind::Invasion), 7);
    }
}

#[test]
fn msh_sentry_alien_invasion_departure_and_blink_use_old_counter_lki_and_never_modify_new_source() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Alien Invasion", &[]);
            g.set_counters(source, Counters::of(CounterKind::Invasion, 2));
            let old = g.object_ref(source);
            begin_combat(&mut g, p);
            g.move_object(source, Zone::Exile);
            if blink {
                g.move_object(source, Zone::Battlefield);
                g.set_counters(source, Counters::of(CounterKind::Invasion, 9));
            }
            assert_ne!(g.object_ref(source), old);
            settle(&mut g);
            let made = tokens(&g, p, "Alien Token");
            assert_eq!(made.len(), 1);
            assert_eq!(g.counter_count(made[0], CounterKind::PlusOne), 2);
            assert_eq!(
                g.counter_count(source, CounterKind::Invasion),
                if blink { 9 } else { 0 }
            );
        }
    }
}

#[test]
fn msh_sentry_alien_invasion_paid_destruction_response_and_control_change_keep_trigger_controller()
{
    for p in [Player::P0, Player::P1] {
        for destroy in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Alien Invasion", &[]);
            g.set_counters(source, Counters::of(CounterKind::Invasion, 2));
            begin_combat(&mut g, p);
            until(&mut g, |g| {
                g.state
                    .stack
                    .last()
                    .is_some_and(|i| matches!(i.kind, StackKind::TriggeredAbility { .. }))
                    && g.state.announcing.is_none()
            });
            if destroy {
                paid(&mut g, p, "Disenchant", &[Target::Card(source)]);
            } else {
                let r = g.object_ref(source);
                g.change_controller(r, p.other());
                settle(&mut g);
            }
            let made = tokens(&g, p, "Alien Token");
            assert_eq!(made.len(), 1);
            assert!(tokens(&g, p.other(), "Alien Token").is_empty());
            assert_eq!(g.counter_count(made[0], CounterKind::PlusOne), 2);
            assert_eq!(
                g.counter_count(source, CounterKind::Invasion),
                if destroy { 0 } else { 3 }
            );
            let history = g.observe(p).counter_placement_history;
            assert!(history.iter().any(|row| row.object == g.object_ref(made[0])
                && row.placed_by == p
                && row.plus_one));
        }
    }
}
