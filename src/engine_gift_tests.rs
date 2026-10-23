//! Paid Gift declaration, delivery ownership, and exact owned continuations.
use super::tests::{scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, z: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, z)
}
fn act(g: &mut Game, action: Action) {
    let p = g.actor().unwrap();
    let before = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap_or_else(|| panic!("{action:?} {:?}", g.observe(p)));
    assert_eq!(g.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(before, g.snapshot());
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(p, rev, index).unwrap();
    g.apply(p, action.clone()).unwrap();
    assert_eq!(child.snapshot(), g.snapshot());
    for q in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(child.observe(q)).unwrap(),
            serde_json::to_value(g.observe(q)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, index), Err(Error::StaleRevision));
    g.apply(p, action).unwrap();
    assert_eq!(after, g.snapshot());
}
fn settle(g: &mut Game) {
    for _ in 0..240 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        act(g, a);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for q in [p, p.other()] {
        g.state.players[q.index()].mana = [20; 6];
        for _ in 0..12 {
            named(&mut g, q, "Forest", Zone::Library);
        }
        named(&mut g, q, "Forest", Zone::Hand);
    }
    g
}
fn paid(g: &mut Game, p: Player, name: &str, promise: bool, target: Option<Target>) -> CardId {
    let card = named(g, p.other(), name, Zone::Exile);
    g.state.exile_permissions.push(ExilePlayPermission {
        object: g.object_ref(card),
        player: p,
        expires_turn: g.state.turn,
    });
    let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
    act(g, Action::AnnounceExile { card });
    assert_eq!(g.state.pending, Pending::Gift { player: p });
    let snap = g.snapshot();
    assert_eq!(
        g.apply(
            p,
            Action::ChooseGift {
                beneficiary: Some(p)
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snap);
    if let Some(target) = target {
        assert_eq!(
            g.apply(p, Action::ChooseTarget { target }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), snap);
    }
    act(
        g,
        Action::ChooseGift {
            beneficiary: promise.then_some(p.other()),
        },
    );
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        }
    }
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, payment);
    assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < mana);
    card
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.is_token()
                && o.card.has_subtype(subtype))
            .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn paid_all_four_gift_kinds_and_decline_foreign_cards_both_seats() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            for route in 0..4 {
                let mut g = setup(p);
                let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let grave = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let initial = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p.other() && o.zone == Zone::Hand)
                    .count();
                let qlife = g.state.players[p.other().index()].life;
                let plife = g.state.players[p.index()].life;
                let (name, target) = match route {
                    0 => ("Blooming Blast", foe),
                    1 => ("Crumb and Get It", own),
                    2 => ("Into the Flood Maw", foe),
                    _ => ("Dewdrop Cure", grave),
                };
                let card = paid(&mut g, p, name, promise, Some(Target::Card(target)));
                settle(&mut g);
                assert_eq!(g.object_view(card).owner, p.other());
                assert_eq!(g.object_view(card).zone, Zone::Graveyard);
                match route {
                    0 => {
                        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
                        assert_eq!(
                            g.state.players[p.other().index()].life,
                            qlife - if promise { 3 } else { 0 }
                        );
                        assert_eq!(
                            tokens(&g, p.other(), "Treasure").len(),
                            usize::from(promise)
                        );
                    }
                    1 => {
                        assert_eq!(g.stats(own).power, 4);
                        assert_eq!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE), promise);
                        assert_eq!(tokens(&g, p.other(), "Food").len(), usize::from(promise));
                    }
                    2 => {
                        assert_eq!(g.object_view(foe).zone, Zone::Hand);
                        let fish = tokens(&g, p.other(), "Fish");
                        assert_eq!(fish.len(), usize::from(promise));
                        if promise {
                            assert!(g.object_view(fish[0]).tapped);
                            assert_eq!(
                                g.stats(fish[0]),
                                Stats {
                                    power: 1,
                                    toughness: 1
                                }
                            );
                        }
                    }
                    _ => {
                        assert_eq!(g.object_view(grave).zone, Zone::Battlefield);
                        assert_eq!(
                            g.state
                                .objects
                                .iter()
                                .filter(|o| o.owner == p.other() && o.zone == Zone::Hand)
                                .count(),
                            initial + usize::from(promise)
                        );
                    }
                }
                assert_eq!(g.state.players[p.index()].life, plife);
            }
        }
    }
}
#[test]
fn permanent_gift_is_separate_from_printed_entry_trigger_and_never_survives_blink() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            let mut g = setup(p);
            let artifact = named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let hand = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p.other() && o.zone == Zone::Hand)
                .count();
            let card = paid(&mut g, p, "Scrapshooter", promise, None);
            settle(&mut g);
            assert_eq!(g.object_view(card).controller, p);
            assert_eq!(g.object_view(card).owner, p.other());
            assert_eq!(
                g.object_view(artifact).zone,
                if promise {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p.other() && o.zone == Zone::Hand)
                    .count(),
                hand + usize::from(promise)
            );
            let before = g.object_ref(card);
            g.move_object(card, Zone::Exile);
            g.move_object(card, Zone::Battlefield);
            assert_ne!(before, g.object_ref(card));
            assert!(g.state.pending_triggers.is_empty());
            assert!(g.state.ordering_triggers.is_empty());
        }
    }
}
fn paid_targets(g: &mut Game, p: Player, name: &str, promise: bool, targets: &[Target]) -> CardId {
    let card = named(g, p.other(), name, Zone::Exile);
    g.state.exile_permissions.push(ExilePlayPermission {
        object: g.object_ref(card),
        player: p,
        expires_turn: g.state.turn,
    });
    act(g, Action::AnnounceExile { card });
    assert_eq!(g.state.pending, Pending::Gift { player: p });
    act(
        g,
        Action::ChooseGift {
            beneficiary: promise.then_some(p.other()),
        },
    );
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    while g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
        let foreign = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.owner == p.other() && o.zone == Zone::Hand)
            .map(|(id, _)| g.object_ref(id as CardId))
            .unwrap();
        let before = g.snapshot();
        assert_eq!(
            g.apply(p, Action::ChooseAdditionalCost { object: foreign }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
            .unwrap();
        act(g, a);
    }
    let before = g.state.players[p.index()].mana.iter().sum::<u32>();
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, a);
    assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < before);
    card
}
#[test]
fn promise_only_widened_targets_and_mandatory_extra_targets_are_declared_before_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enchant = named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let card = named(&mut g, p, "Into the Flood Maw", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card });
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseGift { beneficiary: None })
        );
        act(
            &mut g,
            Action::ChooseGift {
                beneficiary: Some(p.other()),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enchant),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        act(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(enchant).zone, Zone::Hand);
        // Gift precedes the bounce, so the beneficiary's Season still doubles it.
        assert_eq!(tokens(&g, p.other(), "Fish").len(), 2);
        let mut g = setup(p);
        named(&mut g, p, "Forest", Zone::Graveyard);
        let card = named(&mut g, p, "Peerless Recycling", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card });
        assert!(!g.legal_actions().contains(&Action::ChooseGift {
            beneficiary: Some(p.other())
        }));
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseGift { beneficiary: None })
        );
    }
}
#[test]
fn paid_mass_boost_and_damage_gift_branches_both_seats() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            for damage in [false, true] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let own2 = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let life = g.state.players[p.other().index()].life;
                let targets = if promise {
                    vec![if damage {
                        Target::Player(p.other())
                    } else {
                        Target::Card(own)
                    }]
                } else {
                    vec![]
                };
                paid_targets(
                    &mut g,
                    p,
                    if damage {
                        "Wildfire Howl"
                    } else {
                        "Valley Rally"
                    },
                    promise,
                    &targets,
                );
                settle(&mut g);
                if damage {
                    for id in [own, own2, foe] {
                        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                    }
                    assert_eq!(
                        g.state.players[p.other().index()].life,
                        life - if promise { 1 } else { 0 }
                    );
                } else {
                    assert_eq!(g.stats(own).power, 4);
                    assert_eq!(g.stats(own2).power, 4);
                    assert_eq!(g.stats(foe).power, 2);
                    assert_eq!(g.keywords(own).contains(Keywords::FIRST_STRIKE), promise);
                    assert!(!g.keywords(own2).contains(Keywords::FIRST_STRIKE));
                    assert_eq!(tokens(&g, p.other(), "Food").len(), usize::from(promise));
                }
            }
        }
    }
}
#[test]
fn paid_player_and_permanent_protection_is_snapshot_bound_and_gift_ignores_hexproof() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            paid_targets(&mut g, p, "Dawn's Truce", promise, &[]);
            settle(&mut g);
            assert!(g.observe(p.other()).players[p.index()].hexproof_until_end_turn);
            for id in [own, land] {
                assert!(g.keywords(id).contains(Keywords::HEXPROOF));
                assert_eq!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE), promise);
            }
            assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
            let source = g.object_ref(enemy);
            assert!(!g.target_matches(TargetRule::Any, p.other(), source, Target::Player(p)));
            assert!(g.target_matches(TargetRule::Any, p, source, Target::Player(p)));
            let new = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            assert!(!g.keywords(new).contains(Keywords::HEXPROOF));
            if g.actor() != Some(p.other()) {
                act(&mut g, Action::Pass);
            }
            paid_targets(
                &mut g,
                p.other(),
                "Nocturnal Hunger",
                true,
                &[Target::Card(enemy)],
            );
            settle(&mut g);
            assert_eq!(tokens(&g, p, "Food").len(), 1);
            g.finish_cleanup();
            assert!(!g.observe(p).players[p.index()].hexproof_until_end_turn);
            for id in [own, land] {
                assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
                assert!(!g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
            }
        }
    }
}
#[test]
fn paid_spell_return_is_not_countering_and_casting_prohibition_allows_lands_and_mana() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            let mut g = setup(p);
            let q = p.other();
            g.state.active = q;
            g.state.priority = q;
            for actor in [p, q] {
                named(&mut g, actor, "Mountain", Zone::Battlefield);
                named(&mut g, actor, "Shock", Zone::Hand);
            }
            let victim = named(&mut g, q, "Gigantic Big Bear", Zone::Hand);
            assert!(g.object_view(victim).card.cannot_be_countered());
            act(&mut g, Action::AnnounceCast { card: victim });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            act(&mut g, pay);
            act(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p));
            paid_targets(
                &mut g,
                p,
                "Bilbo's Gambit",
                promise,
                &[Target::Spell(victim)],
            );
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert_eq!(g.object_view(victim).owner, q);
            assert_eq!(g.observe(p).casting_prohibited, promise);
            if promise {
                for _ in 0..2 {
                    let actor = g.actor().unwrap();
                    let actions = g.legal_actions();
                    assert!(!actions.iter().any(|a| matches!(
                        a,
                        Action::AnnounceCast { .. }
                            | Action::Cast { .. }
                            | Action::AnnounceExile { .. }
                            | Action::AnnounceAdventure { .. }
                            | Action::AnnounceOmen { .. }
                            | Action::AnnounceWarp { .. }
                            | Action::AnnouncePlotted { .. }
                            | Action::AnnounceFlashback { .. }
                            | Action::AnnounceFlashSurcharge { .. }
                    )));
                    assert!(actions.iter().any(|a| matches!(a, Action::TapMana { .. })));
                    if actor == q {
                        assert!(actions.iter().any(|a| matches!(a, Action::PlayLand { .. })));
                    }
                    act(&mut g, Action::Pass);
                }
                g.finish_cleanup();
                assert!(!g.observe(p).casting_prohibited);
                assert!(
                    g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::AnnounceCast { .. }))
                );
            }
        }
    }
}
#[test]
fn paid_other_complete_gift_consumers_enforce_both_declared_plans_and_cost_ownership() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            for route in 0..8 {
                let mut g = setup(p);
                let q = p.other();
                let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let foe = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
                let big = named(&mut g, q, "Hill Giant", Zone::Battlefield);
                let grave = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let grave2 = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let grave3 = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let hand = g
                    .state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count();
                let (name, targets) = match route {
                    0 => (
                        "Consumed by Greed",
                        if promise {
                            vec![Target::Player(q), Target::Card(grave)]
                        } else {
                            vec![Target::Player(q)]
                        },
                    ),
                    1 => (
                        "Longstalk Brawl",
                        vec![Target::Card(own), Target::Card(foe)],
                    ),
                    2 => (
                        "Mind Spiral",
                        if promise {
                            vec![Target::Player(p), Target::Card(foe)]
                        } else {
                            vec![Target::Player(p)]
                        },
                    ),
                    3 => (
                        "Peerless Recycling",
                        if promise {
                            vec![Target::Card(grave), Target::Card(grave2)]
                        } else {
                            vec![Target::Card(grave)]
                        },
                    ),
                    4 => (
                        "Sazacap's Brew",
                        if promise {
                            vec![Target::Player(p), Target::Card(own)]
                        } else {
                            vec![Target::Player(p)]
                        },
                    ),
                    5 => {
                        let a = named(&mut g, q, "Doubling Season", Zone::Battlefield);
                        let b = named(&mut g, q, "Doubling Season", Zone::Battlefield);
                        (
                            "Wear Down",
                            if promise {
                                vec![Target::Card(a), Target::Card(b)]
                            } else {
                                vec![Target::Card(a)]
                            },
                        )
                    }
                    6 => (
                        "Dewdrop Cure",
                        if promise {
                            vec![
                                Target::Card(grave),
                                Target::Card(grave2),
                                Target::Card(grave3),
                            ]
                        } else {
                            vec![Target::Card(grave), Target::Card(grave2)]
                        },
                    ),
                    _ => {
                        g.state.active = q;
                        g.state.priority = q;
                        let victim = named(&mut g, q, "Hill Giant", Zone::Hand);
                        act(&mut g, Action::AnnounceCast { card: victim });
                        let pay = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::PayCosts { .. }))
                            .unwrap();
                        act(&mut g, pay);
                        act(&mut g, Action::Pass);
                        ("Long River's Pull", vec![Target::Spell(victim)])
                    }
                };
                let spell = paid_targets(&mut g, p, name, promise, &targets);
                settle(&mut g);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(g.object_view(spell).owner, q);
                match route {
                    0 => {
                        assert_eq!(g.object_view(big).zone, Zone::Graveyard);
                        assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
                        assert_eq!(
                            g.object_view(grave).zone,
                            if promise { Zone::Hand } else { Zone::Graveyard }
                        );
                    }
                    1 => {
                        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
                        assert_eq!(
                            g.object_view(own).zone,
                            if promise {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                        if promise {
                            assert_eq!(g.object_view(own).counters.plus_one, 1);
                            assert_eq!(g.object_view(own).damage, 2);
                        }
                    }
                    2 => {
                        assert_eq!(
                            g.state
                                .objects
                                .iter()
                                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                                .count(),
                            hand + 3
                        );
                        assert_eq!(g.object_view(foe).tapped, promise);
                        assert_eq!(g.object_view(foe).counters.stun, u64::from(promise));
                    }
                    3 => {
                        assert_eq!(g.object_view(grave).zone, Zone::Hand);
                        assert_eq!(
                            g.object_view(grave2).zone,
                            if promise { Zone::Hand } else { Zone::Graveyard }
                        );
                    }
                    4 => {
                        assert_eq!(
                            g.state
                                .objects
                                .iter()
                                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                                .count(),
                            hand - 1 + 2
                        );
                        assert_eq!(g.stats(own).power, if promise { 4 } else { 2 });
                    }
                    5 => {
                        for (i, target) in targets.iter().enumerate() {
                            if let Target::Card(id) = target {
                                assert_eq!(g.object_view(*id).zone, Zone::Graveyard, "{i}");
                            }
                        }
                    }
                    6 => {
                        for id in [grave, grave2] {
                            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                        }
                        assert_eq!(
                            g.object_view(grave3).zone,
                            if promise {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                    }
                    _ => {
                        let Target::Spell(id) = targets[0] else {
                            unreachable!()
                        };
                        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                    }
                }
            }
        }
    }
}
fn normal_response(g: &mut Game, p: Player, name: &str, target: Target) -> CardId {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    let card = named(g, p, name, Zone::Hand);
    act(g, Action::AnnounceCast { card });
    act(g, Action::ChooseTarget { target });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, pay);
    card
}
#[test]
fn paid_countered_and_all_illegal_spells_never_give_or_run_other_effects() {
    for p in [Player::P0, Player::P1] {
        for promise in [false, true] {
            for counter in [false, true] {
                let mut g = setup(p);
                let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                let life = g.state.players[p.index()].life;
                let spell =
                    paid_targets(&mut g, p, "Nocturnal Hunger", promise, &[Target::Card(foe)]);
                normal_response(
                    &mut g,
                    p.other(),
                    if counter { "Negate" } else { "Murder" },
                    if counter {
                        Target::Spell(spell)
                    } else {
                        Target::Card(foe)
                    },
                );
                settle(&mut g);
                assert_eq!(tokens(&g, p.other(), "Food").len(), 0);
                assert_eq!(g.state.players[p.index()].life, life);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(
                    g.object_view(foe).zone,
                    if counter {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
        }
    }
}
#[test]
fn paid_partially_illegal_gift_only_target_still_gives_and_draws_without_rebinding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foe = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let before = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand)
            .count();
        paid_targets(
            &mut g,
            p,
            "Mind Spiral",
            true,
            &[Target::Player(p), Target::Card(foe)],
        );
        normal_response(&mut g, p.other(), "Murder", Target::Card(foe));
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count(),
            before + 3
        );
        assert_eq!(tokens(&g, p.other(), "Fish").len(), 1);
    }
}
#[test]
fn actual_paid_source_death_after_entry_binding_preserves_beneficiary_and_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        let enemy = named(&mut g, q, "Doubling Season", Zone::Battlefield);
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == q && o.zone == Zone::Hand)
            .count();
        let source = paid_targets(&mut g, p, "Scrapshooter", true, &[]);
        let cast_object = g.object_ref(source);
        for _ in 0..160 {
            if g.state.pending == Pending::Priority
                && g.object_view(source).zone == Zone::Battlefield
                && g.state.stack.iter().any(|s| s.gift.is_some())
            {
                break;
            }
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            act(&mut g, a);
        }
        let entry = g.object_ref(source);
        let packets = g
            .state
            .stack
            .iter()
            .filter_map(|s| s.gift.as_ref().map(|c| (s.controller, **c)))
            .collect::<Vec<_>>();
        assert_eq!(packets.len(), 2);
        for (controller, c) in packets {
            assert_eq!(controller, p);
            assert_eq!(c.caster, p);
            assert_eq!(c.beneficiary, q);
            assert_eq!(c.source, cast_object);
            assert_eq!(c.entry, Some(entry));
        }
        normal_response(&mut g, q, "Murder", Target::Card(source));
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).owner, q);
        assert_eq!(g.object_view(source).controller, q);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == q && o.zone == Zone::Hand)
                .count(),
            hand + 1
        );
    }
}
#[test]
fn sparse_gift_context_heap_payloads_are_owned_and_counted_once() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Scrapshooter", Zone::Hand);
    let item = g.cast_item(Player::P0, source, [0; 6]);
    g.state.stack.push(item);
    let context = GiftContext {
        source: g.object_ref(source),
        caster: Player::P0,
        beneficiary: Player::P1,
        kind: crate::cards::GiftKind::Card,
        entry: None,
    };
    let before = g.state_bytes();
    g.state.stack.last_mut().unwrap().gift = Some(Box::new(context));
    assert_eq!(g.state_bytes() - before, std::mem::size_of::<GiftContext>());
    let child = {
        let parent = g.clone();
        parent.clone()
    };
    assert_eq!(child.state_bytes(), g.state_bytes());
    assert_eq!(child.snapshot(), g.snapshot());
    let movement = g.entry_move(source, Zone::Battlefield, false);
    let mut batch = entries::EntryBatch {
            temporary_copy: None,
        discard_context: None,
        moves: vec![movement],
        choices: vec![],
        next: 0,
        counter_next: 0,
        entry_bonuses_ready: false,
        counter_order: vec![],
        timestamp_order: Vec::new(),
        timestamps_ready: false,
        resume: None,
        shuffle_after: None,
        replacement_resume: None,
        tap_events: Vec::new(),
        tap_next: 0,
    };
    let before = batch.bytes();
    batch.moves[0].gift = Some(Box::new(context));
    assert_eq!(batch.bytes() - before, std::mem::size_of::<GiftContext>());
    assert_eq!(batch.clone().bytes(), batch.bytes());
}

#[test]
fn paid_gift_damage_after_death_exile_preserves_controller_and_suppresses_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[creature as usize].controller = p.other();
        normal_response(&mut g, p, "Fake Your Own Death", Target::Card(creature));
        settle(&mut g);
        normal_response(&mut g, p, "Giant Growth", Target::Card(creature));
        settle(&mut g);
        assert_eq!(
            (g.stats(creature).power, g.stats(creature).toughness),
            (7, 5)
        );
        let strike = named(&mut g, p, "Pinecone Strike", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: strike });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        for _ in 0..8 {
            if g.state.announcing.is_none() {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            act(&mut g, action);
        }
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(g.object_view(creature).damage, 3);
        assert_eq!(g.observe(p).death_exile_replacements.len(), 1);
        let lives = [g.state.players[0].life, g.state.players[1].life];
        let drawn = g.state.players[p.other().index()].cards_drawn_this_turn;
        let gifts = tokens(&g, p.other(), "Treasure").len();
        let caster_treasures = tokens(&g, p, "Treasure").len();
        paid(
            &mut g,
            p,
            "Blooming Blast",
            true,
            Some(Target::Card(creature)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.object_view(creature).controller, p);
        assert_eq!(g.state.players[p.index()].life, lives[p.index()]);
        assert_eq!(
            g.state.players[p.other().index()].life,
            lives[p.other().index()] - 3
        );
        assert_eq!(
            g.state.players[p.other().index()].cards_drawn_this_turn,
            drawn
        );
        assert!(g.observe(p).death_exile_replacements.is_empty());
        assert!(g.observe(p).temporary_abilities.is_empty());
        assert_eq!(tokens(&g, p, "Treasure").len(), caster_treasures);
        assert_eq!(tokens(&g, p.other(), "Treasure").len(), gifts + 1);
    }
}
