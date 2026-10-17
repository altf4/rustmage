//! Explicit graveyard origins, queued source identity, payments and owned replay.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    // Use the actual printed definitions through ordinary paid actions.
    let card = Card::from_name(name).unwrap();
    let id = spawn(g, p, card, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..120 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::DeclineEffectPayment
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending)
}
fn begin_combat(g: &mut Game) {
    assert_eq!(g.state.step, Step::Main1);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| s.card == id)
        .count()
}
fn dead_phoenix(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, "Flamewake Phoenix", &[]);
    assert!(!g.state.step_sources.contains(&id));
    paid(g, p, "Shock", &[Target::Card(id)]);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    assert_eq!(g.state.step_sources.iter().filter(|&&x| x == id).count(), 1);
    id
}
fn tap(g: &mut Game, id: CardId) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::TapMana { card } | Action::ActivateMana { card, .. } if *card == id)).unwrap();
    checked(g, action);
}
#[test]
fn graveyard_origin_phoenix_actual_red_payment_decline_and_insufficient_mana_both_seats() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let id = dead_phoenix(&mut g, p);
            named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
            let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
            begin_combat(&mut g);
            assert_eq!(count(&g, id), 1);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [1, 0, 0, 0, 0, 0],
                },
            );
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            );
            if pay {
                tap(&mut g, mountain);
                checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 1, 0, 0],
                    },
                );
            } else {
                checked(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(id).zone,
                if pay {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.players[p.index()].mana[3], 0);
            if pay {
                assert!(!g.object_view(id).tapped);
                assert!(
                    g.keywords(id)
                        .contains(Keywords::FLYING.union(Keywords::HASTE))
                );
                assert!(!g.state.step_sources.contains(&id));
                // Returning now has haste and must attack this combat.
                until(&mut g, |g| g.state.pending == Pending::Attackers);
                reject(&mut g, Action::FinishAttackers);
                checked(&mut g, Action::Attack { card: id });
            }
        }
    }
}
#[test]
fn graveyard_origin_phoenix_intervening_ferocious_owner_turn_and_live_power() {
    for p in [Player::P0, Player::P1] {
        for own_turn in [false, true] {
            for power in [3, 4] {
                let mut g = setup(p);
                let id = dead_phoenix(&mut g, p);
                let big = named(
                    &mut g,
                    p,
                    if power == 4 {
                        "Serra Angel"
                    } else {
                        "Hill Giant"
                    },
                    Zone::Battlefield,
                );
                g.state.active = if own_turn { p } else { p.other() };
                g.state.priority = g.state.active;
                begin_combat(&mut g);
                assert_eq!(count(&g, id), usize::from(own_turn && power == 4));
                if own_turn && power == 4 {
                    // An intervening condition is tested again at resolution.
                    g.state.players[p.index()].mana = [30; 6];
                    let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
                    cast(&mut g, bounce, None, &[Target::Card(big)]);
                    settle(&mut g);
                    assert_eq!(g.object_view(big).zone, Zone::Hand);
                    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                    assert_eq!(count(&g, id), 0);
                }
            }
        }
    }
}
#[test]
fn graveyard_origin_phoenix_queued_old_incarnation_payment_does_not_return_new_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = dead_phoenix(&mut g, p);
        named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        begin_combat(&mut g);
        let old = g.object_ref(id);
        // Controlled zone hooks isolate this identity rule from any admitted
        // graveyard-exile card. The queued ability still resolves and may pay.
        g.move_object(id, Zone::Exile);
        assert!(!g.state.step_sources.contains(&id));
        g.move_object(id, Zone::Graveyard);
        assert_ne!(old, g.object_ref(id));
        assert_eq!(g.state.step_sources.iter().filter(|&&x| x == id).count(), 1);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        tap(&mut g, mountain);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana[3], 0);
        g.move_object(id, Zone::Hand);
        assert!(!g.state.step_sources.contains(&id));
    }
}
#[test]
fn graveyard_origin_gate_land_ownership_optional_top_known_identity_and_non_gate() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Gate Colossus", &[]);
            paid(&mut g, p, "Murder", &[Target::Card(id)]);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert!(!g.state.step_sources.contains(&id));
            let forest = named(&mut g, p, "Forest", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: forest });
            assert_eq!(count(&g, id), 0);
            // Event dispatch checks the Gate's controller, not the active seat.
            let enemy_gate = spawn(
                &mut g,
                p.other(),
                Card::from_name("Azorius Guildgate").unwrap(),
                Zone::Hand,
            );
            g.move_object(enemy_gate, Zone::Battlefield);
            assert_eq!(count(&g, id), 0);
            g.state.land_played = false;
            let gate = named(&mut g, p, "Azorius Guildgate", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: gate });
            assert_eq!(count(&g, id), 1);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            let old = g.object_ref(id);
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            if accept {
                assert_eq!(g.object_view(id).zone, Zone::Library);
                assert_eq!(g.state.players[p.index()].library.last(), Some(&id));
                assert_ne!(g.object_ref(id), old);
                // Returning a public source to its owner's top is known to both.
                for player in [p, p.other()] {
                    assert_eq!(
                        g.observe(player).known_library_top[p.index()]
                            .as_ref()
                            .map(|v| v.object),
                        Some(g.object_ref(id))
                    );
                }
            } else {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn graveyard_origin_gate_exact_source_owner_and_multiple_queued_packets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Gate Colossus", &[]);
        let gate = named(&mut g, p, "Azorius Guildgate", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: gate });
        assert_eq!(count(&g, id), 0); // Battlefield text has no graveyard trigger.
        // Actual control-changing spell followed by destruction: a graveyard
        // ability belongs to its owner, independently of its former controller.
        next_turn(&mut g);
        until(&mut g, |g| g.state.step == Step::Main1);
        g.state.players[p.other().index()].mana = [30; 6];
        paid(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(id)],
        );
        assert_eq!(g.object_view(id).controller, p.other());
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).controller, p);
        let first = named(&mut g, p, "Azorius Guildgate", Zone::Hand);
        let second = named(&mut g, p, "Boros Guildgate", Zone::Hand);
        // One simultaneous entry batch verifies both observers see the same
        // source incarnation; the second old packet cannot follow the top move.
        let events = g.move_objects_raw(&[first, second], Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        assert_eq!(count(&g, id), 2);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        let top = g.object_ref(id);
        assert_eq!(g.object_view(id).zone, Zone::Library);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_ref(id), top);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&id));
        assert_ne!(g.state.players[p.other().index()].library.last(), Some(&id));
    }
}

#[test]
fn graveyard_origin_removed_battlefield_abilities_recover_and_coexist_with_battlefield_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let phoenix = paid(&mut g, p, "Flamewake Phoenix", &[]);
        let demon = paid(&mut g, p, "Desecration Demon", &[]);
        paid(&mut g, p, "Stop Cold", &[Target::Card(phoenix)]);
        assert!(!g.keywords(phoenix).contains(Keywords::FLYING));
        assert!(!g.printed_abilities_available(phoenix));
        paid(&mut g, p, "Murder", &[Target::Card(phoenix)]);
        assert!(g.object_view(phoenix).printed_abilities_available());
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        begin_combat(&mut g);
        assert_eq!(count(&g, phoenix), 1);
        assert_eq!(count(&g, demon), 1);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        tap(&mut g, mountain);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(phoenix).zone, Zone::Battlefield);
        assert!(g.keywords(phoenix).contains(Keywords::FLYING));
        // An explicit graveyard observer must never resurrect a legacy
        // battlefield event on the same source merely by being scanned there.
        g.move_object(demon, Zone::Graveyard);
        let source = g.object_view(demon);
        assert!(!g.queue_trigger(&source, 0));
        assert_eq!(count(&g, demon), 0);
    }
}
