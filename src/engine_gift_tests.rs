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
