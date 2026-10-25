//! Paid whole-card scenarios; production phase-boundary probes are named explicitly.
use super::assurance::big_helpers::{checked, named, next_turn, setup};
use super::*;
const SPIDER: &str = "Spider-Man, To the Rescue";
const SPEED: &str = "Speed, Young Avenger";
fn game(p: Player) -> Game {
    let mut g = setup(p);
    for player in &mut g.state.players {
        player.mana = [0; 6];
    }
    g
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
#[track_caller]
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..400 {
        if stop(g) {
            return;
        }
        let a = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            Action::ChooseOptional { accept: false }
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    ) || matches!(a, Action::FinishTargets)
                        && g.state.announcing.as_ref().is_some_and(|a| a.spec.max == 0)
                })
                .unwrap_or_else(|| {
                    panic!("unexpected {:?}, {:?}", g.state.pending, g.legal_actions())
                })
        };
        checked(g, a);
    }
    panic!("reflexive scenario did not progress");
}
fn settle(g: &mut Game) {
    until(g, quiet);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game, packet: u64) {
    for _ in 0..15 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("packet stalled");
}
fn announce(g: &mut Game, p: Player, name: &str, mana: Mana, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        checked(g, a);
    }
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g, packet);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = announce(g, p, name, mana, &[]);
    settle(g);
    id
}
fn spider(g: &mut Game, p: Player) -> ObjectRef {
    let id = announce(g, p, SPIDER, [0, 0, 0, 0, 1, 2], &[]);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    g.object_ref(id)
}
fn payment(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
}
fn shock(g: &mut Game, p: Player, mana: Mana) {
    announce(g, p, "Shock", mana, &[Target::Player(p.other())]);
    payment(g);
}
fn child(g: &mut Game, source: ObjectRef) -> u64 {
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.object == source && a.item.kind == StackKind::TriggeredAbility { ability: 1 }
        })
    });
    g.state.announcing.as_ref().unwrap().item.id
}
fn target(g: &mut Game, packet: u64, id: CardId) {
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    finish(g, packet);
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(raw.snapshot(), before);
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn protection(g: &mut Game, p: Player, id: CardId) -> CardId {
    let aura = named(g, p, "Witness Protection", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
    aura
}
fn haste(g: &mut Game, id: CardId) {
    let source = g.object_ref(id);
    g.grant_keyword_effects(&[id], source, Keywords::HASTE);
}
fn rule(g: &Game, id: CardId) -> bool {
    g.keywords(id).contains(Keywords::ONLY_HASTE_BLOCKERS)
}
fn speed_rule(g: &mut Game, p: Player, source: ObjectRef, id: CardId) {
    shock(g, p, [0, 0, 0, 1, 0, 1]);
    checked(
        g,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    let packet = child(g, source);
    target(g, packet, id);
    settle(g);
}
#[test]
fn msh_reflexive_spider_hybrid_paid_decline_and_no_early_target() {
    for p in [Player::P0, Player::P1] {
        for mana in [[1, 0, 0, 0, 0, 2], [0, 0, 0, 0, 1, 2]] {
            let mut g = game(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = announce(&mut g, p, SPIDER, mana, &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 2));
            assert!(
                g.keywords(id).contains(
                    Keywords::FLASH
                        .union(Keywords::REACH)
                        .union(Keywords::VIGILANCE)
                )
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
            checked(&mut g, Action::ChooseOptional { accept: false });
            settle(&mut g);
            assert!(!g.object_view(id).tapped);
            assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn msh_reflexive_spider_success_target_rules_owned_and_real_destroy_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opponent = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let source = spider(&mut g, p);
        let parent = g.state.resolving.as_ref().unwrap().item.id;
        g = orphan(g, Action::ChooseOptional { accept: true });
        assert!(g.object_view(source.card).tapped);
        let packet = child(&mut g, source);
        assert_ne!(packet, parent);
        for id in [source.card, opponent, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g, packet);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        announce(
            &mut g,
            p.other(),
            "Murder",
            [0, 0, 2, 0, 0, 1],
            &[Target::Card(bear)],
        );
        settle(&mut g);
        assert!(g.on_battlefield(bear));
        next_turn(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn msh_reflexive_spider_source_failure_before_offer_and_during_optional_is_exact() {
    for p in [Player::P0, Player::P1] {
        for failure in 0..3 {
            let mut g = game(p);
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = spider(&mut g, p);
            // A production phase-boundary probe changes the source while the owned
            // optional menu is pending. These moves are not player priority actions.
            if failure == 0 {
                g.tap_objects(&[source.card]);
            } else {
                g.move_object(source.card, Zone::Exile);
                if failure == 2 {
                    g.move_object(source.card, Zone::Battlefield);
                }
            }
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert!(!g.state.roles.as_ref().is_some_and(|r| {
                r.keyword_effects
                    .iter()
                    .any(|r| r.keywords.contains(Keywords::INDESTRUCTIBLE))
            }));
            if failure == 2 {
                assert_ne!(g.object_ref(source.card), source);
                assert!(!g.object_view(source.card).tapped);
            }
        }
    }
}
#[test]
fn msh_reflexive_spider_tapped_entry_replacement_and_departure_before_resolution_skip_offer() {
    for p in [Player::P0, Player::P1] {
        for gone in [false, true] {
            let mut g = game(p);
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            if !gone {
                named(
                    &mut g,
                    p.other(),
                    "Authority of the Consuls",
                    Zone::Battlefield,
                );
            }
            let id = announce(&mut g, p, SPIDER, [1, 0, 0, 0, 0, 2], &[]);
            until(&mut g, |g| {
                g.on_battlefield(id) && g.state.resolving.is_none()
            });
            if gone {
                g.move_object(id, Zone::Exile);
            } else {
                assert!(g.object_view(id).tapped);
            }
            settle(&mut g);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|s| s.kind == StackKind::TriggeredAbility { ability: 1 })
            );
            assert!(!g.state.roles.as_ref().is_some_and(|r| {
                r.keyword_effects
                    .iter()
                    .any(|r| r.keywords.contains(Keywords::INDESTRUCTIBLE))
            }));
        }
    }
}
#[test]
fn msh_reflexive_spider_control_changes_and_source_departure_preserve_child_controller() {
    for p in [Player::P0, Player::P1] {
        for become_land in [false, true] {
            let mut g = game(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let theirs = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = spider(&mut g, p);
            g.record_control(source, p.other(), None);
            if become_land {
                let aura = named(
                    &mut g,
                    p.other(),
                    "Imprisoned in the Moon",
                    Zone::Battlefield,
                );
                g.state.objects[aura as usize].attached_to = Some(source);
                assert!(!g.is_creature(source.card));
            }
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseOptional { accept: true });
            let packet = child(&mut g, source);
            assert_eq!(g.state.announcing.as_ref().unwrap().item.controller, p);
            assert!(g.object_view(source.card).tapped);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(theirs),
                },
            );
            g.move_object(source.card, Zone::Exile);
            target(&mut g, packet, bear);
            settle(&mut g);
            assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

#[test]
fn msh_reflexive_spider_new_incarnation_is_another_and_old_child_ignores_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = spider(&mut g, p);
        protection(&mut g, p.other(), source.card);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let packet = child(&mut g, source);
        g.move_object(source.card, Zone::Exile);
        g.move_object(source.card, Zone::Battlefield);
        let new_ref = g.object_ref(source.card);
        assert_ne!(source, new_ref);
        target(&mut g, packet, source.card);
        settle(&mut g);
        assert!(g.keywords(source.card).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.object_view(source.card).tapped);
    }
}
#[test]
fn msh_reflexive_spider_target_rechecks_current_attack_control_type_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        for change in 0..4 {
            let mut g = game(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = spider(&mut g, p);
            checked(&mut g, Action::ChooseOptional { accept: true });
            let packet = child(&mut g, source);
            target(&mut g, packet, bear);
            // Production state probes recheck the bound target at resolution;
            // the separate flash/combat group tests natural declaration timing.
            match change {
                0 => g.state.attackers.push(Attacker {
                    card: bear,
                    defender: Some(CombatDefender::Player(p.other())),
                    blocked: false,
                }),
                1 => g.record_control(g.object_ref(bear), p.other(), None),
                2 => {
                    g.move_object(bear, Zone::Exile);
                    g.move_object(bear, Zone::Battlefield);
                }
                _ => {
                    let aura = named(
                        &mut g,
                        p.other(),
                        "Imprisoned in the Moon",
                        Zone::Battlefield,
                    );
                    g.state.objects[aura as usize].attached_to = Some(g.object_ref(bear));
                }
            }
            settle(&mut g);
            assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
            assert!(g.object_view(source.card).tapped);
        }
    }
}
#[test]
fn msh_reflexive_spider_blocker_target_flash_reach_and_vigilance_paid_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = spider(&mut g, p);
        // Blocking, unlike attacking, is legal for the protected creature.
        g.state.blocking.push(bear);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let packet = child(&mut g, source);
        target(&mut g, packet, bear);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        g.state.blocking.clear();
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: source.card });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(source.card).tapped);
        settle(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.pending == Pending::Attackers
        });
        let flyer = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        g.state.objects[flyer as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: flyer });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: source.card,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.object_view(flyer).zone, Zone::Graveyard);
        // Pay a second physical Spider-Man during opponent priority, before it resolves.
        let old = g.object_ref(source.card);
        g.move_object(source.card, Zone::Exile);
        let flash = announce(&mut g, p, SPIDER, [1, 0, 0, 0, 0, 2], &[]);
        settle(&mut g);
        assert!(g.on_battlefield(flash));
        assert_ne!(g.object_ref(flash), old);
    }
}
#[test]
fn msh_reflexive_spider_real_tap_observer_and_no_target_still_pays() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let source = spider(&mut g, p);
        let bow = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
        g.state.objects[bow as usize].attached_to = Some(source);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert!(g.object_view(source.card).tapped);
        assert!(g.first_tap(source));
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn msh_reflexive_speed_paid_haste_and_optional_payment_has_no_early_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (2, 2));
        assert!(g.keywords(id).contains(Keywords::HASTE));
        shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(!rule(&g, id));
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        until(&mut g, |g| {
            g.state.pending == Pending::Attackers && g.state.active == p
        });
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn msh_reflexive_speed_real_mana_window_and_shortage_owned_success() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let source = g.object_ref(id);
        shock(&mut g, p, [0, 0, 0, 1, 0, 0]);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
        );
        let land = named(&mut g, p, "Mountain", Zone::Battlefield);
        checked(&mut g, Action::TapMana { card: land });
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        let packet = child(&mut g, source);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        finish(&mut g, packet);
        settle(&mut g);
        assert!(rule(&g, id));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(land).tapped);
    }
}
#[test]
fn msh_reflexive_speed_keyword_target_legality_and_later_loss_fizzles() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let source = g.object_ref(id);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        let packet = child(&mut g, source);
        for bad in [bear, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        haste(&mut g, bear);
        target(&mut g, packet, bear);
        protection(&mut g, p.other(), bear);
        settle(&mut g);
        assert!(!rule(&g, bear));
    }
}
#[test]
fn msh_reflexive_speed_rule_survives_recipient_ability_loss_and_live_blocker_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let source = g.object_ref(id);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        speed_rule(&mut g, p, source, id);
        assert!(rule(&g, id));
        let aura = protection(&mut g, p.other(), id);
        assert!(!g.keywords(id).contains(Keywords::HASTE));
        assert!(rule(&g, id));
        assert!(!g.can_block_pair(blocker, id));
        haste(&mut g, blocker);
        assert!(g.can_block_pair(blocker, id));
        let remove = protection(&mut g, p, blocker);
        assert!(!g.can_block_pair(blocker, id));
        g.move_object(remove, Zone::Graveyard);
        assert!(g.can_block_pair(blocker, id));
        g.move_object(aura, Zone::Graveyard);
        assert!(rule(&g, id));
        next_turn(&mut g);
        assert!(!rule(&g, id));
    }
}
#[test]
fn msh_reflexive_speed_other_players_target_control_source_departure_and_blink_cleanup() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = game(p);
            let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
            let source = g.object_ref(id);
            let target_id = named(&mut g, p.other(), "Slickshot Show-Off", Zone::Battlefield);
            shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
            g.record_control(source, p.other(), None);
            checked(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            let packet = child(&mut g, source);
            assert_eq!(g.state.announcing.as_ref().unwrap().item.controller, p);
            target(&mut g, packet, target_id);
            g.move_object(id, Zone::Exile);
            settle(&mut g);
            assert!(rule(&g, target_id));
            if change == 0 {
                g.record_control(g.object_ref(target_id), p, None);
                assert!(rule(&g, target_id));
            } else if change == 1 {
                g.move_object(target_id, Zone::Exile);
                g.move_object(target_id, Zone::Battlefield);
                assert!(!rule(&g, target_id));
            } else {
                next_turn(&mut g);
                assert!(!rule(&g, target_id));
            }
        }
    }
}
#[test]
fn msh_reflexive_speed_suppression_spell_filter_and_stack_copy_does_not_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let source = g.object_ref(id);
        paid(&mut g, p, "Bear Cub", [0, 0, 0, 0, 1, 1]);
        announce(
            &mut g,
            p.other(),
            "Shock",
            [0, 0, 0, 1, 0, 0],
            &[Target::Player(p)],
        );
        settle(&mut g);
        let aura = protection(&mut g, p.other(), id);
        announce(
            &mut g,
            p,
            "Shock",
            [0, 0, 0, 1, 0, 0],
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        g.move_object(aura, Zone::Graveyard);
        shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
        let suppression = protection(&mut g, p.other(), id);
        let other = named(&mut g, p, "Slickshot Show-Off", Zone::Battlefield);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        let packet = child(&mut g, source);
        target(&mut g, packet, other);
        g.move_object(suppression, Zone::Graveyard);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell)
            .unwrap()
            .id;
        let mut context = g
            .state
            .stack
            .iter()
            .find(|s| s.object == source && s.kind == StackKind::TriggeredAbility { ability: 1 })
            .unwrap()
            .clone();
        context.targets = vec![g.bind_target(Target::Stack(original))];
        // Bounded production copy invocation: allocate a real spell-copy object,
        // not a cloned physical spell or a newly admitted copy card.
        assert!(!g.copy_target_spell(&context, 0, false, crate::cards::CopyExceptions::NONE));
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.card != context.card)
        );
        settle(&mut g);
        assert!(rule(&g, other));
    }
}
#[test]
fn msh_reflexive_targets_retarget_and_copy_independently_after_real_paid_actions() {
    for p in [Player::P0, Player::P1] {
        for use_spider in [false, true] {
            let mut g = game(p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let invalid = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = if use_spider {
                let s = spider(&mut g, p);
                checked(&mut g, Action::ChooseOptional { accept: true });
                s
            } else {
                let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
                haste(&mut g, a);
                haste(&mut g, b);
                shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
                checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
                g.object_ref(id)
            };
            let packet = child(&mut g, source);
            target(&mut g, packet, a);
            let child_id = g.state.stack.iter().find(|s| s.id == packet).unwrap().id;
            announce(
                &mut g,
                p.other(),
                "Bolt Bend",
                [0, 0, 0, 1, 0, 3],
                &[Target::Stack(child_id)],
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Retarget { .. })
            });
            reject(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(invalid),
                },
            );
            g = orphan(
                g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(b),
                },
            );
            checked(&mut g, Action::FinishStackTargets);
            let mut copy = g
                .state
                .stack
                .iter()
                .find(|s| s.id == child_id)
                .unwrap()
                .clone();
            copy.id = g.next_stack_id();
            copy.is_copy = true;
            copy.targets = vec![g.bind_target(Target::Card(a))];
            g.state.stack.push(copy);
            settle(&mut g);
            for id in [a, b] {
                assert!(if use_spider {
                    g.keywords(id).contains(Keywords::INDESTRUCTIBLE)
                } else {
                    rule(&g, id)
                });
            }
        }
    }
}
#[test]
fn msh_reflexive_private_decisions_preserve_full_hidden_observations_and_legal_actions() {
    for p in [Player::P0, Player::P1] {
        for tap in [false, true] {
            let mut g = game(p);
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let secret = named(&mut g, p.other(), "Murder", Zone::Hand);
            let top = named(&mut g, p, "Shock", Zone::Library);
            if tap {
                spider(&mut g, p);
            } else {
                paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
                shock(&mut g, p, [0, 0, 0, 1, 0, 1]);
            }
            let mut hidden = g.clone();
            hidden.state.objects[secret as usize].card = Card::from_name("Giant Growth").unwrap();
            hidden.state.objects[top as usize].card = Card::from_name("Forest").unwrap();
            assert_eq!(
                serde_json::to_value(g.observe(p)).unwrap(),
                serde_json::to_value(hidden.observe(p)).unwrap()
            );
            // The opponent sees its own changed hand; compare its view with only the hidden library changed.
            hidden.state.objects[secret as usize].card = g.state.objects[secret as usize].card;
            assert_eq!(views(&g), views(&hidden));
            assert_eq!(g.legal_actions(), hidden.legal_actions());
        }
    }
}

#[test]
fn msh_reflexive_speed_restricted_lots_cannot_pay_trigger_or_enable_a_mana_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let hideout = named(&mut g, p, "Villainous Hideout", Zone::Battlefield);
        checked(
            &mut g,
            Action::ActivateMana {
                card: hideout,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        g.state.players[p.index()].mana[3] += 1;
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g, packet);
        payment(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
        );
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(!rule(&g, id));
    }
}
#[test]
fn msh_reflexive_spider_copied_parent_only_taps_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = spider(&mut g, p);
        // Copy the real initial ability before resolving either copy. The first
        // succeeds; the other sees the same exact permanent already tapped.
        let mut copied_parent = g.state.resolving.as_ref().unwrap().item.clone();
        copied_parent.id = g.next_stack_id();
        copied_parent.is_copy = true;
        g.state.stack.push(copied_parent);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let packet = child(&mut g, source);
        target(&mut g, packet, bear);
        settle(&mut g);
        assert!(g.first_tap(source));
        assert_eq!(
            g.observe(p)
                .keyword_effects
                .iter()
                .filter(|r| r.recipient == g.object_ref(bear)
                    && r.keywords.contains(Keywords::INDESTRUCTIBLE))
                .count(),
            1
        );
    }
}

#[test]
fn msh_reflexive_spider_torpor_orb_suppresses_only_entry_trigger_not_paid_body() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let orb = paid(&mut g, p, "Torpor Orb", [0, 0, 0, 0, 0, 2]);
        let source = paid(&mut g, p, SPIDER, [1, 0, 0, 0, 0, 2]);
        assert!(g.on_battlefield(source));
        assert!(!g.object_view(source).tapped);
        assert!(
            g.keywords(source).contains(
                Keywords::FLASH
                    .union(Keywords::REACH)
                    .union(Keywords::VIGILANCE)
            )
        );
        assert!(
            g.state
                .roles
                .as_ref()
                .is_none_or(|r| r.keyword_effects.is_empty())
        );
        g.move_object(orb, Zone::Graveyard);
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
    }
}
#[test]
fn msh_reflexive_speed_actual_block_declaration_checks_haste_then_keeps_legal_block() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let id = paid(&mut g, p, SPEED, [0, 0, 0, 1, 0, 1]);
        let source = g.object_ref(id);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        speed_rule(&mut g, p, source, id);
        until(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        haste(&mut g, blocker);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        // A layer change after the committed declaration does not undo that block.
        protection(&mut g, p, blocker);
        assert!(!g.keywords(blocker).contains(Keywords::HASTE));
        until(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(g.on_battlefield(id));
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_reflexive_spider_flash_after_real_attack_rejects_attacking_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let attacker = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let waiting = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[attacker as usize].summoning_sick = false;
        until(&mut g, |g| {
            g.state.pending == Pending::Attackers && g.state.active == p
        });
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        let source = spider(&mut g, p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let packet = child(&mut g, source);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        target(&mut g, packet, waiting);
        settle(&mut g);
        assert!(g.keywords(waiting).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(attacker).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn msh_reflexive_spider_granted_indestructible_obeys_later_layer_six_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = spider(&mut g, p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let packet = child(&mut g, source);
        target(&mut g, packet, bear);
        settle(&mut g);
        let aura = protection(&mut g, p.other(), bear);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        protection(&mut g, p.other(), bear);
        announce(
            &mut g,
            p.other(),
            "Murder",
            [0, 0, 2, 0, 0, 1],
            &[Target::Card(bear)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}
