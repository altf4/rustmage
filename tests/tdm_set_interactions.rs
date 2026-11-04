//! Set-only outcomes composing independently verified actual TDM definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut owned = {
        let parent = g.clone();
        parent.clone()
    };
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), saved);
    owned.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), owned.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(owned.observe(viewer)).unwrap()
        );
    }
    g.restore(&saved).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), owned.snapshot());
}
fn drain(g: &mut Game, target: Option<CardId>) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let targeted = target.map(|card| Action::ChooseTarget {
            target: Target::Card(card),
        });
        let action = targeted
            .filter(|a| actions.contains(a))
            .or_else(|| {
                actions.into_iter().find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::FinishStackTargets
                            | Action::ChooseReplacement { .. }
                    )
                })
            })
            .unwrap_or_else(|| {
                panic!(
                    "explicit interaction choice required: {:?}",
                    g.state.pending
                )
            });
        checked(g, action);
    }
    panic!("96-action interaction budget");
}
fn renew(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, payment);
}
fn treasures(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype("Treasure"))
                .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn f02_renew_funds_renew_mixed_kinds_and_finality_controls_host_death_transfer() {
    for p in [Player::P0, Player::P1] {
        for graveyard_cast in [false, true] {
            let mut g = scenario(p);
            let veteran = put(
                &mut g,
                p,
                "Hundred-Battle Veteran",
                if graveyard_cast {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                },
            );
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            cast(&mut g, veteran, None);
            drain(&mut g, None);
            assert_eq!(stats(&g, veteran), (4, 2));
            assert_eq!(
                g.counter_count(veteran, CounterKind::Finality),
                u64::from(graveyard_cast)
            );
            for name in [
                "Kheru Goldkeeper",
                "Kishla Skimmer",
                "Host of the Hereafter",
            ] {
                g.state.players[p.index()].mana = [8; 6];
                let card = put(&mut g, p, name, Zone::Hand);
                cast(&mut g, card, None);
                drain(&mut g, None);
            }
            let goldkeeper = g
                .state
                .objects
                .iter()
                .position(|o| {
                    o.zone == Zone::Battlefield && o.card.identity() == "Kheru Goldkeeper"
                })
                .unwrap() as CardId;
            let host = g
                .state
                .objects
                .iter()
                .position(|o| {
                    o.zone == Zone::Battlefield && o.card.identity() == "Host of the Hereafter"
                })
                .unwrap() as CardId;
            assert_eq!(g.counter_count(host, CounterKind::PlusOne), 2);
            assert_eq!(
                stats(&g, veteran),
                (4, 2),
                "two +1 counters still represent only one kind"
            );
            let champion = put(&mut g, p, "Champion of Dusan", Zone::Graveyard);
            let assistant = put(&mut g, p, "Alchemist's Assistant", Zone::Graveyard);
            let library = g.library_card_count(p);
            let hand = g.hand_count(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            renew(&mut g, champion, veteran);
            assert_eq!(g.object_view(champion).zone, Zone::Exile);
            drain(&mut g, None);
            assert_eq!(g.counter_count(veteran, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(veteran, CounterKind::Trample), 1);
            assert_eq!(
                stats(&g, veteran),
                if graveyard_cast { (7, 7) } else { (5, 3) }
            );
            assert_eq!(g.library_card_count(p), library - 1);
            assert_eq!(g.hand_count(p), hand + 1);
            let treasure = treasures(&g);
            assert_eq!(treasure.len(), 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            checked(&mut g, Action::TapMana { card: forest });
            checked(
                &mut g,
                Action::ActivateMana {
                    card: treasure[0],
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 1, 0, 0, 0],
                },
            );
            assert_ne!(g.object_view(treasure[0]).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 1, 0]);
            renew(&mut g, assistant, veteran);
            drain(&mut g, None);
            assert_eq!(g.object_view(assistant).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.counter_count(veteran, CounterKind::Lifelink), 1);
            assert!(g
                .keywords(veteran)
                .contains(Keywords::TRAMPLE.union(Keywords::LIFELINK)));
            assert_eq!(stats(&g, veteran), (7, 7));
            assert_eq!(
                treasures(&g).len(),
                1,
                "each departure creates a Treasure; first was actually spent"
            );
            assert_eq!(
                g.library_card_count(p),
                library - 1,
                "Skimmer only draws once this turn"
            );
            assert_eq!(g.hand_count(p), hand + 1);
            let prey = put(&mut g, p, "Dragon's Prey", Zone::Hand);
            g.state.players[p.index()].mana = [8; 6];
            cast(&mut g, prey, Some(Target::Card(veteran)));
            drain(&mut g, Some(goldkeeper));
            assert_eq!(
                g.object_view(veteran).zone,
                if graveyard_cast {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            for kind in [
                CounterKind::PlusOne,
                CounterKind::Trample,
                CounterKind::Lifelink,
            ] {
                assert_eq!(
                    g.counter_count(goldkeeper, kind),
                    u64::from(!graveyard_cast),
                    "Host only sees an actual death"
                );
            }
            assert_eq!(g.counter_count(host, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(goldkeeper, CounterKind::Finality), 0);
        }
    }
}

fn resolve_loot(g: &mut Game, discard: CardId) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let selected = Action::Discard { card: discard };
        if actions.contains(&selected) {
            checked(g, selected);
        } else {
            let action = actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::FinishStackTargets
                            | Action::ChooseReplacement { .. }
                    )
                })
                .unwrap_or_else(|| panic!("explicit loot choice required: {:?}", g.state.pending));
            act(g, action);
        }
    }
    panic!("96-action loot interaction budget");
}

#[test]
fn f07_omen_shuffle_does_not_return_dragonstorm_but_later_dragon_entry_repeats_loot() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let omen = put(&mut g, p, "Marang River Regent", Zone::Hand);
        let dragon = put(&mut g, p, "Marang River Regent", Zone::Hand);
        let storm = put(&mut g, p, "Roiling Dragonstorm", Zone::Hand);
        let library = g.library_card_count(p);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, storm, None);
        resolve_loot(&mut g, discard);
        assert_eq!(g.object_view(storm).zone, Zone::Battlefield);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.library_card_count(p), library - 2);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let second_discard = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.owner == p && o.zone == Zone::Hand && o.card == Card::Forest)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceOmen { card: omen });
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let chosen = g.state.stack.last().unwrap().source.unwrap().card;
        assert_eq!(chosen.name(), "Coil and Catch");
        assert!(chosen.has_subtype("Omen"));
        assert!(!chosen.has_subtype("Dragon"));
        resolve_loot(&mut g, second_discard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(omen).zone, Zone::Library);
        assert_eq!(
            g.object_view(omen).card,
            Card::from_name("Marang River Regent").unwrap()
        );
        assert_eq!(g.library_card_count(p), library - 4);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.object_view(storm).zone, Zone::Battlefield);
        g.state.players[p.index()].mana = [8; 6];
        reject(&mut g, Action::AnnounceCast { card: omen });
        reject(&mut g, Action::AnnounceOmen { card: omen });
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, dragon, None);
        drain(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
        assert_eq!(stats(&g, dragon), (6, 7));
        assert!(g.keywords(dragon).contains(Keywords::FLYING));
        assert_eq!(g.object_view(omen).zone, Zone::Library);
        assert_eq!(g.object_view(storm).zone, Zone::Hand);
        let old_storm = g.object_ref(storm);
        let third_discard = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.owner == p && o.zone == Zone::Hand && o.card == Card::Forest)
                    .then_some(id as CardId)
            })
            .unwrap();
        let library = g.library_card_count(p);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, storm, None);
        resolve_loot(&mut g, third_discard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(storm).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(storm), old_storm);
        assert_eq!(g.library_card_count(p), library - 2);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.object_view(third_discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
    }
}

#[test]
fn f06_dragon_witness_mox_stays_locked_until_fresh_start_suppresses_clarion() {
    for p in [Player::P0, Player::P1] {
        for remove_aura in [false, true] {
            let mut g = scenario(p);
            let mox = put(&mut g, p, "Mox Jasper", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            cast(&mut g, mox, None);
            drain(&mut g, None);
            let produce = Action::ActivateMana {
                card: mox,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            };
            reject(&mut g, produce.clone());
            put(&mut g, p.other(), "Marang River Regent", Zone::Battlefield);
            reject(&mut g, produce.clone());
            let clarion = put(&mut g, p, "Clarion Conqueror", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
            cast(&mut g, clarion, None);
            drain(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.has_subtype(clarion, "Dragon"));
            reject(&mut g, produce.clone());
            assert!(!g.object_view(mox).tapped);
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            act(&mut g, Action::TapMana { card: forest });
            assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 1, 0]);
            let aura = put(&mut g, p, "Fresh Start", Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(clarion)));
            drain(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, clarion), (-2, 3));
            assert!(!g.keywords(clarion).contains(Keywords::FLYING));
            assert!(g.has_subtype(clarion, "Dragon"));
            assert!(g.legal_actions().contains(&produce));
            if remove_aura {
                for _ in 0..2 {
                    let land = put(&mut g, p, "Forest", Zone::Battlefield);
                    act(&mut g, Action::TapMana { card: land });
                }
                let reclamation = put(&mut g, p, "Heritage Reclamation", Zone::Hand);
                act(&mut g, Action::AnnounceCast { card: reclamation });
                act(&mut g, Action::ChooseMode { mode: 1 });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(aura),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
                drain(&mut g, None);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
                assert_eq!(g.object_view(reclamation).zone, Zone::Graveyard);
                assert_eq!(stats(&g, clarion), (3, 3));
                assert!(g.keywords(clarion).contains(Keywords::FLYING));
                assert!(!g.object_view(mox).tapped);
                reject(&mut g, produce);
            } else {
                checked(&mut g, produce);
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
                assert!(g.object_view(mox).tapped);
            }
        }
    }
}

#[test]
fn f14_storm_counts_both_players_actual_casts_and_permanent_copies_bypass_token_doubling() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
        g.set_counters(elspeth, Counters::of(CounterKind::Loyalty, 5));
        let own_dragon = put(&mut g, p, "Marang River Regent", Zone::Battlefield);
        let enemy_dragon = put(&mut g, p.other(), "Marang River Regent", Zone::Battlefield);
        let mox = put(&mut g, p, "Mox Jasper", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        cast(&mut g, mox, None);
        drain(&mut g, None);
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        let reclamation = put(&mut g, p.other(), "Heritage Reclamation", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 1];
        act(&mut g, Action::AnnounceCast { card: reclamation });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mox),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        drain(&mut g, None);
        assert_eq!(g.object_view(mox).zone, Zone::Graveyard);
        assert_eq!(g.object_view(reclamation).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
        assert_eq!(g.actor(), Some(p));
        let original = put(&mut g, p, "Stormscale Scion", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 4];
        cast(&mut g, original, None);
        let captured = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .find_map(|item| item.copy_event.as_deref())
            .expect("actual captured Storm event");
        assert_eq!(captured.amount, 2);
        checked(&mut g, Action::Pass);
        drain(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let scions: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::from_name("Stormscale Scion").unwrap())
                .then_some(id as CardId)
            })
            .collect();
        assert_eq!(
            scions.len(),
            3,
            "two earlier actual casts, across both players"
        );
        assert_eq!(
            scions
                .iter()
                .filter(|&&id| g.object_view(id).is_token())
                .count(),
            2,
            "resolved permanent spell copies bypass actual Elspeth token doubling"
        );
        assert!(!g.object_view(original).is_token());
        for &id in &scions {
            assert_eq!(stats(&g, id), (6, 6));
            assert!(g.keywords(id).contains(Keywords::FLYING));
        }
        assert_eq!(stats(&g, own_dragon), (9, 10));
        assert_eq!(stats(&g, enemy_dragon), (6, 7));
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 1);
        assert_eq!(g.object_view(elspeth).zone, Zone::Battlefield);
        g.move_object(original, Zone::Graveyard);
        assert_eq!(stats(&g, own_dragon), (8, 9));
        for id in scions.into_iter().filter(|&id| id != original) {
            assert_eq!(stats(&g, id), (5, 5));
        }
        assert_eq!(stats(&g, enemy_dragon), (6, 7));
    }
}
