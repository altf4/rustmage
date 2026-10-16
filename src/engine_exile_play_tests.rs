use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 15] = [
    "Alania's Pathmaker",
    "Blazing Crescendo",
    "Clockwork Percussionist",
    "Cori Mountain Monastery",
    "Count on Luck",
    "Crimson Operative",
    "Diversion Specialist",
    "Emberheart Challenger",
    "Equilibrium Adept",
    "Gundabad Opportunist",
    "Kulrath Zealot",
    "Outlaws' Fury",
    "Seize Opportunity",
    "Sizzling Changeling",
    "Zuko, Exiled Prince",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    spawn(g, p, Card::from_name(n).unwrap(), z)
}
fn replay(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    assert!(
        g.legal_actions().contains(&a),
        "illegal {a:?} pending {:?} stack {:?}",
        g.state.pending,
        g.state.stack
    );
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    for _ in 0..100 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseMode { mode: 0 }
                        | Action::ChooseOffspring { pay: false }
                        | Action::ChooseSpellCost { cost: 0 }
                        | Action::ChooseActivationCost { cost: 0 }
                        | Action::ChooseForageRoute {
                            route: forage::ForageRoute::Black
                        }
                )
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseTarget { .. }
                            | Action::FinishTargets
                            | Action::ChooseAdditionalCost { .. }
                            | Action::ChooseForage { .. }
                            | Action::PayCosts { .. }
                    )
                })
            })
            .unwrap_or_else(|| panic!("announcement: {actions:?}"));
        replay(g, a.clone());
    }
    panic!("announcement stuck");
}
fn resolve(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        if g.state.announcing.is_some() {
            finish(g);
            continue;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::PutTrigger { .. } | Action::FinishSearch
                    )
                })
            })
            .cloned()
            .unwrap_or_else(|| panic!("{:?}", g.legal_actions()));
        replay(g, a);
    }
    panic!("resolution stuck");
}
fn exile(g: &mut Game, p: Player, n: &str, next: bool) -> CardId {
    let id = named(g, p, n, Zone::Library);
    g.exile_top_play(p, 1, next);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
    id
}
#[test]
fn all_fifteen_complete_bodies_grant_real_exact_public_permissions_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            g.state.players[p.index()].mana = [30; 6];
            let payload = named(&mut g, p, "Shock", Zone::Library);
            let old = g.object_ref(payload);
            let source = named(
                &mut g,
                p,
                name,
                if matches!(i, 1 | 11 | 12) {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            match i {
                0 | 5 | 8 | 9 | 10 => {
                    g.move_object(source, Zone::Hand);
                    replay(&mut g, Action::AnnounceCast { card: source });
                    finish(&mut g);
                }
                1 | 11 | 12 => {
                    named(&mut g, p, "Swab Goblin", Zone::Battlefield);
                    replay(&mut g, Action::AnnounceCast { card: source });
                    finish(&mut g);
                }
                2 | 13 => {
                    g.move_object(source, Zone::Graveyard);
                    g.give_priority();
                }
                3 | 6 | 14 => {
                    if i == 6 {
                        named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    }
                    replay(
                        &mut g,
                        Action::AnnounceActivate {
                            card: source,
                            ability: if i == 3 { 1 } else { 0 },
                            counter_payment: Counters::default(),
                        },
                    );
                    finish(&mut g);
                }
                4 => {
                    g.begin_step(Step::Upkeep);
                    g.give_priority();
                }
                7 => {
                    let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
                    replay(&mut g, Action::AnnounceCast { card: spell });
                    replay(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(source),
                        },
                    );
                    finish(&mut g);
                }
                _ => unreachable!(),
            }
            resolve(&mut g);
            assert_eq!(
                g.state.objects[payload as usize].zone,
                Zone::Exile,
                "{name}"
            );
            let reference = g.object_ref(payload);
            assert_eq!(reference.generation, old.generation + 1);
            let rights = &g.observe(p.other()).exile_permissions;
            let right = rights.iter().find(|r| r.object == reference).unwrap();
            assert_eq!(right.player, p);
            assert_eq!(
                right.expires_turn,
                if matches!(i, 4 | 6 | 7 | 14) { 3 } else { 5 }
            );
            if i == 12 {
                assert_eq!(rights.len(), 2);
            } else {
                assert_eq!(rights.len(), 1);
            }
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.reference == reference)
            );
            if i == 5 {
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 3,
                        toughness: 2
                    }
                );
            }
            if i == 7 {
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 6,
                        toughness: 6
                    }
                );
            }
            // Permission does not depend on the source staying on the battlefield.
            if g.on_battlefield(source) {
                g.move_object(source, Zone::Graveyard);
                g.give_priority();
                resolve(&mut g);
            }
            assert!(g.can_play_exiled(p, payload));
            g.state.priority = p;
            replay(&mut g, Action::AnnounceExile { card: payload });
            let item = &g.state.announcing.as_ref().unwrap().item;
            assert_eq!(item.cast.unwrap().origin, Zone::Exile);
            assert_eq!(item.cast.unwrap().method, CastMethod::Normal);
            assert!(!g.state.objects[payload as usize].flashback);
            finish(&mut g);
            resolve(&mut g);
            assert_eq!(g.state.objects[payload as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn exact_permissions_land_limits_timing_owner_generation_and_no_direct_bypass() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let land = exile(&mut g, Player::P0, "Forest", true);
    let sorcery = exile(&mut g, Player::P0, "Divination", true);
    let instant = exile(&mut g, Player::P0, "Shock", true);
    let old = g.snapshot();
    assert_eq!(
        g.apply(Player::P1, Action::AnnounceExile { card: instant }),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), old);
    reject(&mut g, Action::AnnounceCast { card: instant });
    reject(
        &mut g,
        Action::Cast {
            card: instant,
            target: Some(Target::Player(Player::P1)),
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    reject(
        &mut g,
        Action::BeginCast {
            card: sorcery,
            payment: [0, 1, 0, 0, 0, 2],
        },
    );
    replay(&mut g, Action::PlayLand { card: land });
    assert_eq!(g.state.objects[land as usize].zone, Zone::Battlefield);
    assert!(!g.can_play_exiled(Player::P0, land));
    let second = exile(&mut g, Player::P0, "Island", true);
    reject(&mut g, Action::PlayLand { card: second });
    g.state.active = Player::P1;
    reject(&mut g, Action::AnnounceExile { card: sorcery });
    reject(&mut g, Action::PlayLand { card: second });
    replay(&mut g, Action::AnnounceExile { card: instant });
    finish(&mut g);
    resolve(&mut g);
    g.state.active = Player::P0;
    g.state.priority = Player::P0;
    let reference = g.object_ref(sorcery);
    g.move_object(sorcery, Zone::Hand);
    g.move_object(sorcery, Zone::Exile);
    assert_ne!(g.object_ref(sorcery), reference);
    reject(&mut g, Action::AnnounceExile { card: sorcery });
    assert!(!g.can_play_exiled(Player::P0, sorcery));
    g.state.priority = Player::P1;
    reject(&mut g, Action::AnnounceExile { card: second });
    assert_eq!(
        g.observe(Player::P0).exile_permissions,
        g.observe(Player::P1).exile_permissions
    );
}
#[test]
fn cleanup_deadlines_cover_both_active_players_end_and_repeated_cleanup_grants() {
    for p in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            for step in [Step::Main1, Step::End, Step::Cleanup] {
                let mut g = scenario();
                g.state.active = active;
                g.state.priority = p;
                g.state.step = step;
                let short = exile(&mut g, p, "Shock", false);
                let long = exile(&mut g, p, "Think Twice", true);
                let deadline = if active == p { 5 } else { 4 };
                assert_eq!(
                    g.state
                        .exile_permissions
                        .iter()
                        .find(|x| x.object.card == long)
                        .unwrap()
                        .expires_turn,
                    deadline
                );
                g.finish_cleanup();
                assert!(!g.can_play_exiled(p, short));
                assert!(g.can_play_exiled(p, long));
                g.state.turn = deadline;
                g.state.active = p;
                g.state.step = Step::End;
                assert!(g.can_play_exiled(p, long));
                g.finish_cleanup();
                assert!(!g.can_play_exiled(p, long));
                // A grant in a priority-bearing repeated cleanup is available until the next
                // cleanup pass, not until some later turn's cleanup.
                g.state.step = Step::Cleanup;
                let fresh = exile(&mut g, p, "Shock", false);
                assert!(g.can_play_exiled(p, fresh));
                g.finish_cleanup();
                assert!(!g.can_play_exiled(p, fresh));
            }
        }
    }
}
#[test]
fn ordinary_exile_cast_departures_do_not_acquire_flashback_replacement() {
    for destination in [Zone::Graveyard, Zone::Hand, Zone::Library, Zone::Exile] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        let id = exile(&mut g, Player::P0, "Think Twice", true);
        replay(&mut g, Action::AnnounceExile { card: id });
        finish(&mut g);
        g.state.stack.clear();
        let event = g.move_object(id, destination);
        assert_eq!(event.destination, destination);
        assert!(g.state.exile_permissions.is_empty());
        assert!(!g.state.objects[id as usize].flashback);
    }
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    g.state.players[1].mana = [0, 2, 0, 0, 0, 0];
    let id = exile(&mut g, Player::P0, "Think Twice", true);
    replay(&mut g, Action::AnnounceExile { card: id });
    finish(&mut g);
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: counter });
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
}
#[test]
fn exile_staged_costs_hybrids_offspring_discard_forage_and_reductions() {
    for name in [
        "Laughing Mad",
        "Defibrillating Current",
        "Coruscation Mage",
        "Fell",
    ] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let id = exile(&mut g, Player::P0, name, true);
        replay(&mut g, Action::AnnounceExile { card: id });
        finish(&mut g);
        resolve(&mut g);
        assert_eq!(
            g.state.objects[id as usize].zone,
            if name == "Coruscation Mage" {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
    let mut g = scenario();
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    let id = exile(&mut g, Player::P0, "Tome Blast", true);
    replay(&mut g, Action::AnnounceExile { card: id });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert!(g.legal_actions().contains(&Action::PayCosts {
        payment: [0, 0, 0, 1, 0, 0]
    }));
    finish(&mut g);
    resolve(&mut g);
    // A mandatory forage-or-black spell is still not free merely because it is exiled.
    let name = crate::Card::ALL
        .iter()
        .copied()
        .find(|c| {
            c.additional_cost()
                .is_some_and(|c| c.kind == EffectChoiceKind::ForageOrBlack)
        })
        .unwrap();
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let id = exile(&mut g, Player::P0, name.name(), true);
    replay(&mut g, Action::AnnounceExile { card: id });
    finish(&mut g);
    resolve(&mut g);
    assert!(g.state.exile_permissions.is_empty());
}
#[test]
fn deadlines_generation_and_stack_capacity_failures_are_atomic() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = exile(&mut g, Player::P0, "Think Twice", true);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P0, Action::AnnounceExile { card: id }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert!(g.can_play_exiled(Player::P0, id));
    g.state.next_stack_id = 0;
    g.state.objects[id as usize].generation = u64::MAX;
    g.state.exile_permissions[0].object = g.object_ref(id);
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P0, Action::AnnounceExile { card: id }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Alania's Pathmaker", Zone::Hand);
    g.state.players[0].mana = [30; 6];
    replay(&mut g, Action::AnnounceCast { card: id });
    finish(&mut g);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    g.state.turn = u32::MAX;
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P1, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
}
#[test]
fn coris_entry_replacement_outlaw_guard_and_remaining_modal_boost_are_complete() {
    for basic in [None, Some("Mountain"), Some("Plains"), Some("Island")] {
        let mut g = scenario();
        if let Some(n) = basic {
            named(&mut g, Player::P0, n, Zone::Battlefield);
        }
        let id = exile(&mut g, Player::P0, "Cori Mountain Monastery", true);
        replay(&mut g, Action::PlayLand { card: id });
        assert_eq!(
            g.state.objects[id as usize].tapped,
            !matches!(basic, Some("Plains" | "Island"))
        );
    }
    for outlaw in [false, true] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        let creature = named(
            &mut g,
            Player::P0,
            if outlaw { "Swab Goblin" } else { "Bear Cub" },
            Zone::Battlefield,
        );
        let base = g.stats(creature);
        let id = named(&mut g, Player::P0, "Outlaws' Fury", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        finish(&mut g);
        resolve(&mut g);
        assert_eq!(
            g.stats(creature),
            Stats {
                power: base.power + 2,
                toughness: base.toughness
            }
        );
        assert_eq!(g.state.exile_permissions.len(), usize::from(outlaw));
    }
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Seize Opportunity", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseMode { mode: 1 });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 4,
            toughness: 3
        }
    );
    assert!(g.state.exile_permissions.is_empty());
}
#[test]
fn all_two_brid_equivalents_and_paid_offspring_from_exile_preserve_printed_values() {
    let c = Card::from_name("Defibrillating Current").unwrap();
    for (index, cost) in c.cost_options().enumerate() {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let id = exile(&mut g, Player::P0, c.name(), true);
        replay(&mut g, Action::AnnounceExile { card: id });
        replay(&mut g, Action::ChooseSpellCost { cost: index as u16 });
        assert_eq!(g.observe(Player::P0).announcing_cost, Some(cost));
        finish(&mut g);
        let cast = g.state.stack.last().unwrap();
        assert_eq!(cast.source_card.fixed_mana_value(), 6);
        resolve(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    }
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = exile(&mut g, Player::P0, "Coruscation Mage", true);
    replay(&mut g, Action::AnnounceExile { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .cast
            .unwrap()
            .offspring,
        Some(true)
    );
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|v| v.zone == Zone::Battlefield && v.card.name() == "Coruscation Mage")
            .count(),
        2
    );
    assert!(g.state.exile_permissions.is_empty());
}
#[test]
fn forage_exile_three_food_and_black_routes_remain_real_costs_from_exile() {
    for route in [
        ForageRoute::ExileThree,
        ForageRoute::Food,
        ForageRoute::Black,
    ] {
        let mut g = scenario();
        g.state.players[0].mana = [30; 6];
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let grave: Vec<_> = (0..3)
            .map(|_| named(&mut g, Player::P0, "Forest", Zone::Graveyard))
            .collect();
        let factory = Card::catalogue()
            .find(|c| c.is_token() && c.token_info().unwrap().name == "Food Token")
            .unwrap();
        let food = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
        let c = Card::ALL
            .iter()
            .copied()
            .find(|c| {
                c.additional_cost()
                    .is_some_and(|a| a.kind == EffectChoiceKind::ForageOrBlack)
            })
            .unwrap();
        let id = exile(&mut g, Player::P0, c.name(), true);
        replay(&mut g, Action::AnnounceExile { card: id });
        replay(&mut g, Action::ChooseForageRoute { route });
        while g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseForage { .. }))
        {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseForage { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        finish(&mut g);
        resolve(&mut g);
        for card in grave {
            assert_eq!(
                g.state.objects[card as usize].zone,
                if route == ForageRoute::ExileThree {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
        assert_eq!(
            g.state.objects[food as usize].zone,
            if route == ForageRoute::Food {
                Zone::Outside
            } else {
                Zone::Battlefield
            }
        );
        assert!(g.state.exile_permissions.is_empty());
    }
}
#[test]
fn valiant_once_flurry_second_spell_kulrath_cycling_and_firebending_residuals() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let source = named(
        &mut g,
        Player::P0,
        "Emberheart Challenger",
        Zone::Battlefield,
    );
    for turn_cast in 0..2 {
        let id = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        finish(&mut g);
        resolve(&mut g);
        assert_eq!(g.state.exile_permissions.len(), 1);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 6 + turn_cast * 4,
                toughness: 6 + turn_cast * 4
            }
        );
    }
    let source = named(&mut g, Player::P0, "Equilibrium Adept", Zone::Battlefield);
    g.state.players[0].spells_cast_this_turn = 0;
    for n in 1..=2 {
        let id = named(&mut g, Player::P0, "Shock", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        finish(&mut g);
        resolve(&mut g);
        assert_eq!(g.keywords(source).contains(Keywords::DOUBLE_STRIKE), n == 2);
    }
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = named(&mut g, Player::P0, "Kulrath Zealot", Zone::Hand);
    let rng = g.state.rng;
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    finish(&mut g);
    resolve(&mut g);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    assert!(g.state.exile_permissions.is_empty());
    assert_ne!(g.state.rng, rng);
    assert_eq!(g.hand_count(Player::P0), 1);
    assert_eq!(
        g.observe(Player::P1)
            .objects
            .iter()
            .filter(|v| v.zone == Zone::Hand)
            .count(),
        1
    );
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Zuko, Exiled Prince", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: id });
    replay(&mut g, Action::FinishAttackers);
    resolve(&mut g);
    assert_eq!(g.state.players[0].combat_red_mana, 3);
    assert_eq!(g.state.players[0].mana[3], 3);
}
#[test]
fn exile_life_choice_land_does_not_pollute_known_hand_and_empty_grants_are_pure() {
    let mut g = scenario();
    let id = exile(&mut g, Player::P0, "Watery Grave", true);
    replay(&mut g, Action::PlayLand { card: id });
    assert!(g.observe(Player::P1).entry_life.is_some());
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert!(g.state.known_hand.is_empty());
    assert!(!g.state.objects[id as usize].tapped);
    assert_eq!(g.state.players[0].life, 18);
    assert!(g.state.exile_permissions.is_empty());
    let mut g = scenario();
    let ids = g.state.players[0].library.to_vec();
    for id in ids {
        g.move_object(id, Zone::Graveyard);
    }
    let rng = g.state.rng;
    g.exile_top_play(Player::P0, 2, true);
    assert!(g.state.exile_permissions.is_empty());
    assert_eq!(g.state.rng, rng);
}
#[test]
fn failed_multi_card_grant_and_expired_spell_or_land_actions_are_atomic() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let id = named(&mut g, Player::P0, "Seize Opportunity", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseMode { mode: 0 });
    finish(&mut g);
    let library = g.state.players[0].library.to_vec();
    let second = library[library.len() - 2];
    g.state.objects[second as usize].generation = u64::MAX;
    replay(&mut g, Action::Pass);
    let old = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(Player::P1, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.revision(), rev);
    assert!(g.state.exile_permissions.is_empty());
    assert_eq!(g.state.players[0].library.as_slice(), library.as_slice());
    for next in [false, true] {
        let mut g = scenario();
        let spell = exile(&mut g, Player::P0, "Think Twice", next);
        let land = exile(&mut g, Player::P0, "Forest", next);
        let deadline = g.state.exile_permissions[0].expires_turn;
        g.state.turn = deadline;
        g.finish_cleanup();
        g.state.priority = Player::P0;
        g.state.active = Player::P0;
        g.state.step = Step::Main1;
        g.state.players[0].mana = [30; 6];
        reject(&mut g, Action::AnnounceExile { card: spell });
        reject(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
    }
}
#[test]
fn actual_cleanup_priority_grant_expires_on_next_cleanup_pass_not_next_turn() {
    let mut g = scenario();
    g.state.step = Step::Cleanup;
    g.state.players[0].mana = [30; 6];
    let dying = named(
        &mut g,
        Player::P0,
        "Clockwork Percussionist",
        Zone::Battlefield,
    );
    g.state.objects[dying as usize].counters.minus_one = 1;
    g.state.objects[dying as usize].growth.toughness = 1;
    let challenger = named(
        &mut g,
        Player::P0,
        "Emberheart Challenger",
        Zone::Battlefield,
    );
    let short = named(&mut g, Player::P0, "Shock", Zone::Library);
    named(&mut g, Player::P0, "Shock", Zone::Library);
    g.finish_cleanup();
    assert_eq!(g.state.turn, 3);
    assert_eq!(g.state.step, Step::Cleanup);
    resolve(&mut g);
    assert_eq!(g.state.step, Step::Cleanup);
    let growth = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: growth });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(challenger),
        },
    );
    finish(&mut g);
    resolve(&mut g);
    assert!(g.can_play_exiled(Player::P0, short));
    assert_eq!(
        g.state
            .exile_permissions
            .iter()
            .find(|r| r.object.card == short)
            .unwrap()
            .expires_turn,
        3
    );
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceExile { card: short })
    );
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.turn, 4);
    assert!(!g.can_play_exiled(Player::P0, short));
    assert_eq!(g.state.exile_permissions.len(), 1); // Clockwork's next-turn right survives.
}
#[test]
fn blazing_fizzles_and_seize_zero_one_two_targets_revalidate_independently() {
    let mut g = scenario();
    g.state.players[0].mana = [30; 6];
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Blazing Crescendo", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(&mut g);
    let library = g.state.players[0].library.to_vec();
    g.move_object(bear, Zone::Graveyard);
    resolve(&mut g);
    assert!(g.state.exile_permissions.is_empty());
    assert_eq!(g.state.players[0].library.as_slice(), library.as_slice());
    assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    for count in 0..=2 {
        for removed in 0..=count {
            let mut g = scenario();
            g.state.players[0].mana = [30; 6];
            let bears = [
                named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield),
                named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield),
            ];
            let id = named(&mut g, Player::P0, "Seize Opportunity", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            replay(&mut g, Action::ChooseMode { mode: 1 });
            for card in bears.iter().take(count) {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(*card),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                replay(&mut g, Action::FinishTargets);
            }
            finish(&mut g);
            for card in bears.iter().take(removed) {
                g.move_object(*card, Zone::Graveyard);
            }
            resolve(&mut g);
            for (j, card) in bears.iter().enumerate() {
                if j < removed {
                    assert_eq!(g.state.objects[*card as usize].zone, Zone::Graveyard);
                } else {
                    assert_eq!(
                        g.stats(*card),
                        if j < count {
                            Stats {
                                power: 4,
                                toughness: 3,
                            }
                        } else {
                            Stats {
                                power: 2,
                                toughness: 2,
                            }
                        }
                    );
                }
            }
            assert!(g.state.exile_permissions.is_empty());
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn cori_reserves_its_tap_source_and_exile_cast_triggers_only_after_real_payment() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Cori Mountain Monastery",
        Zone::Battlefield,
    );
    let mut lands: Vec<_> = (0..3)
        .map(|_| named(&mut g, Player::P0, "Forest", Zone::Battlefield))
        .collect();
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    lands.push(named(&mut g, Player::P0, "Mountain", Zone::Battlefield));
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::TapMana{card}|Action::ActivateMana{card,..}|Action::AnnounceMana{card,..} if *card==source)));
    reject(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    for card in lands {
        replay(&mut g, Action::TapMana { card });
    }
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 3, 0],
        },
    );
    assert!(g.state.objects[source as usize].tapped);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    resolve(&mut g);
    assert_eq!(g.state.exile_permissions.len(), 1);
    let mut g = scenario();
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    let observer = named(&mut g, Player::P0, "Coruscation Mage", Zone::Battlefield);
    let spell = exile(&mut g, Player::P0, "Shock", true);
    replay(&mut g, Action::AnnounceExile { card: spell });
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert!(g.state.pending_triggers.is_empty());
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert!(
        g.state
            .stack
            .iter()
            .any(|s| s.card == observer && matches!(s.kind, StackKind::TriggeredAbility { .. }))
    );
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Stack);
    resolve(&mut g);
    assert_eq!(g.state.players[1].life, 17);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
}
