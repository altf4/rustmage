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
