//! Paid casting choices, simultaneous sacrifice and private raid selection.
use super::ability_timeline_tests::{checked, next_turn, setup};
use super::tests::spawn;
use super::*;

// Scenario insertion intentionally permits gated definitions: admission is enabled
// only after these tests prove their behavior through ordinary legal actions.
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
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
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
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
                        Action::PayCosts { .. } | Action::FinishTargets | Action::PutTrigger { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn paid(g: &mut Game) {
    until(g, |g| g.state.announcing.is_none());
}
fn settle(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn mana(g: &Game, p: Player) -> u32 {
    g.state.players[p.index()].mana.iter().sum()
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}
fn edict_board(g: &mut Game, p: Player, own: usize, enemy: usize) -> [Vec<CardId>; 2] {
    [
        (0..own)
            .map(|_| named(g, p, "Bear Cub", Zone::Battlefield))
            .collect(),
        (0..enemy)
            .map(|_| named(g, p.other(), "Bear Cub", Zone::Battlefield))
            .collect(),
    ]
}
#[test]
fn fdn_cast_choices_edict_alternative_checks_all_creatures_and_printed_cost_stays_available() {
    for p in [Player::P0, Player::P1] {
        for total in [12, 13] {
            let mut g = setup(p);
            edict_board(&mut g, p, 4, total - 4);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            let id = named(&mut g, p, "Blasphemous Edict", Zone::Hand);
            assert_eq!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: id }),
                total == 13
            );
            if total == 12 {
                reject(&mut g, Action::AnnounceCast { card: id });
                continue;
            }
            checked(&mut g, Action::AnnounceCast { card: id });
            reject(&mut g, Action::ChooseSpellCost { cost: 0 });
            checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            paid(&mut g);
            assert_eq!(mana(&g, p), 0);
            let packet = g.state.stack.last().unwrap();
            assert_eq!(packet.mana_value(), 5);
            assert_eq!(packet.cast.unwrap().mana_spent(), 1);
        }
        let mut g = setup(p);
        let id = named(&mut g, p, "Blasphemous Edict", Zone::Hand);
        let before = mana(&g, p);
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(&mut g, Action::ChooseSpellCost { cost: 1 });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        paid(&mut g);
        settle(&mut g);
        assert_eq!(before - mana(&g, p), 5);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn fdn_cast_choices_edict_thirteen_each_commit_simultaneously_and_ignore_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = edict_board(&mut g, p, 14, 3);
        let shield = named(&mut g, p, "Divine Resilience", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: shield });
        checked(&mut g, Action::ChooseKicker { pay: false });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(ids[0][0]),
            },
        );
        paid(&mut g);
        settle(&mut g);
        assert!(g.keywords(ids[0][0]).contains(Keywords::INDESTRUCTIBLE));
        let id = named(&mut g, p, "Blasphemous Edict", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        paid(&mut g);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Sacrifice { .. }))
        });
        let active = g.state.active;
        assert_eq!(g.actor(), Some(active));
        for seat in [active, active.other()] {
            let group = if seat == p { &ids[0] } else { &ids[1] };
            let foreign = if seat == p { ids[1][0] } else { ids[0][13] };
            let object = g.object_ref(foreign);
            reject(&mut g, Action::Sacrifice { object });
            for (i, &chosen) in group.iter().take(13).enumerate() {
                assert_eq!(g.actor(), Some(seat));
                let object = g.object_ref(chosen);
                checked(&mut g, Action::Sacrifice { object });
                if seat == active || i + 1 < group.len().min(13) {
                    assert!(
                        ids.iter()
                            .flatten()
                            .all(|&id| g.object_view(id).zone == Zone::Battlefield)
                    );
                }
            }
        }
        settle(&mut g);
        assert_eq!(
            ids[0]
                .iter()
                .filter(|&&id| g.object_view(id).zone == Zone::Battlefield)
                .count(),
            1
        );
        assert!(
            ids[0][..13]
                .iter()
                .chain(&ids[1])
                .all(|&id| g.object_view(id).zone == Zone::Graveyard)
        );
    }
}
#[test]
fn fdn_cast_choices_edict_locks_condition_before_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids = edict_board(&mut g, p, 1, 12);
        g.state.players[p.index()].mana = [0; 6];
        let swamp = named(&mut g, p, "Swamp", Zone::Battlefield);
        let id = named(&mut g, p, "Blasphemous Edict", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        // The alternative was chosen and total locked before mana abilities.
        g.move_object(ids[0][0], Zone::Graveyard);
        checked(&mut g, Action::TapMana { card: swamp });
        paid(&mut g);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 1);
    }
}
#[test]
fn fdn_cast_choices_eaten_alive_sacrifice_or_mana_and_both_target_kinds() {
    for p in [Player::P0, Player::P1] {
        for extra_mana in [false, true] {
            for own_target in [false, true] {
                let mut g = setup(p);
                let target = named(
                    &mut g,
                    if own_target { p } else { p.other() },
                    "Bear Cub",
                    Zone::Battlefield,
                );
                let sacrifice = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let id = named(&mut g, p, "Eaten Alive", Zone::Hand);
                let before = mana(&g, p);
                checked(&mut g, Action::AnnounceCast { card: id });
                checked(
                    &mut g,
                    Action::ChooseSpellCost {
                        cost: u16::from(extra_mana),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if !extra_mana {
                    let object = g.object_ref(sacrifice);
                    checked(&mut g, Action::ChooseAdditionalCost { object });
                }
                paid(&mut g);
                assert_eq!(before - mana(&g, p), if extra_mana { 5 } else { 1 });
                assert_eq!(
                    g.object_view(sacrifice).zone,
                    if extra_mana {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.state.stack.last().unwrap().mana_value(), 1);
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Exile);
            }
        }
        let mut g = setup(p);
        let target = named(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        g.state.objects[target as usize].counters.loyalty = 6;
        let id = named(&mut g, p, "Eaten Alive", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        paid(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
    }
}
#[test]
fn fdn_cast_choices_eaten_alive_sacrificed_target_fizzles_and_foreign_cost_rejected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let id = named(&mut g, p, "Eaten Alive", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(&mut g, Action::ChooseSpellCost { cost: 1 });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        for bad in [enemy, land] {
            let object = g.object_ref(bad);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(target);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        paid(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
    }
}
#[test]
fn fdn_cast_choices_eaten_alive_empty_board_requires_full_mana_and_valid_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, "Eaten Alive", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        paid(&mut g);
        // A new incarnation is not the old target even in the same zone.
        g.move_object(enemy, Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
    }
}
#[test]
fn fdn_cast_choices_divine_kicker_changes_cardinality_pays_cost_and_expires() {
    for p in [Player::P0, Player::P1] {
        for kick in [false, true] {
            let mut g = setup(p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, "Divine Resilience", Zone::Hand);
            let before = mana(&g, p);
            checked(&mut g, Action::AnnounceCast { card: id });
            checked(&mut g, Action::ChooseKicker { pay: kick });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if !kick {
                reject(&mut g, Action::FinishTargets);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            if kick {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            }
            paid(&mut g);
            assert_eq!(before - mana(&g, p), if kick { 4 } else { 1 });
            settle(&mut g);
            assert!(g.keywords(a).contains(Keywords::INDESTRUCTIBLE));
            assert_eq!(g.keywords(b).contains(Keywords::INDESTRUCTIBLE), kick);
            assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
            next_turn(&mut g);
            assert!(!g.keywords(a).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(b).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn fdn_cast_choices_divine_zero_targets_only_when_kicked_and_partly_stale_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Divine Resilience", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(&mut g, Action::ChooseKicker { pay: false });
        checked(&mut g, Action::ChooseKicker { pay: true });
        // No candidates: the optional target group finishes automatically.
        paid(&mut g);
        settle(&mut g);
        assert_eq!(mana(&g, p), 0);
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let id = named(&mut g, p, "Divine Resilience", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseKicker { pay: true });
        for card in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        paid(&mut g);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.keywords(a).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.keywords(b).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn fdn_cast_choices_gutless_paid_raid_private_optional_top_three() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].attacked_this_turn = true;
            let mut top = vec![];
            for name in ["Plains", "Bear Cub", "Opt"].into_iter().rev() {
                top.push(named(&mut g, p, name, Zone::Library));
            }
            top.reverse();
            let outsider = named(&mut g, p.other(), "Shivan Dragon", Zone::Library);
            let id = named(&mut g, p, "Gutless Plunderer", Zone::Hand);
            let before = mana(&g, p);
            checked(&mut g, Action::AnnounceCast { card: id });
            paid(&mut g);
            assert_eq!(before - mana(&g, p), 3);
            until(&mut g, |g| g.observe(p).library_selection.is_some());
            assert_eq!(g.object_view(id).power, 2);
            assert_eq!(g.object_view(id).toughness, 2);
            assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(
                view.remaining
                    .iter()
                    .map(|v| v.reference.card)
                    .collect::<Vec<_>>(),
                top
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            assert!(top.iter().all(|&id| !visible(&g, p.other(), id)));
            assert!(!visible(&g, p, outsider));
            reject(&mut g, Action::SelectLibrary { card: outsider });
            let original = g.state.players[p.index()].library.clone();
            if choose {
                checked(&mut g, Action::SelectLibrary { card: top[1] });
                reject(&mut g, Action::SelectLibrary { card: top[0] });
                assert_eq!(g.state.players[p.index()].library, original);
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            for (i, &id) in top.iter().enumerate() {
                assert_eq!(
                    g.object_view(id).zone,
                    if choose && i == 1 {
                        Zone::Library
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            if choose {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&top[1]));
            }
        }
    }
}
#[test]
fn fdn_cast_choices_gutless_no_raid_recheck_short_library_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for raid in [false, true] {
            for count in [0, 1, 2] {
                let mut g = setup(p);
                let old = g.state.players[p.index()].library.clone();
                for id in old {
                    g.move_object(id, Zone::Exile);
                }
                let tops = (0..count)
                    .map(|_| named(&mut g, p, "Bear Cub", Zone::Library))
                    .collect::<Vec<_>>();
                g.state.players[p.index()].attacked_this_turn = raid;
                let id = named(&mut g, p, "Gutless Plunderer", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: id });
                paid(&mut g);
                until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
                g.move_object(id, Zone::Graveyard);
                if raid && count > 0 {
                    until(&mut g, |g| g.observe(p).library_selection.is_some());
                    checked(&mut g, Action::FinishLibrarySelection);
                }
                settle(&mut g);
                assert!(tops.iter().all(|&id| g.object_view(id).zone
                    == if raid { Zone::Graveyard } else { Zone::Library }));
            }
        }
        let mut g = setup(p);
        g.state.players[p.index()].attacked_this_turn = true;
        let id = named(&mut g, p, "Gutless Plunderer", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        paid(&mut g);
        until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
        let before = g.state.players[p.index()].library.clone();
        g.state.players[p.index()].attacked_this_turn = false;
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library, before);
    }
}
