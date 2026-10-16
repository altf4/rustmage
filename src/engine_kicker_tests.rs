use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Burst Lightning",
    "Firebending Lesson",
    "Gatekeeper of Malakir",
    "Grow from the Ashes",
    "Into the Roil",
    "Whoosh!",
    "Nullpriest of Oblivion",
    "Sun-Blessed Healer",
    "Zuko's Conviction",
    "Gnarlid Colony",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(fork.snapshot(), e);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn choose(g: &mut Game, target: Option<Target>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|t| {
            actions
                .iter()
                .find(|a| **a == Action::ChooseTarget { target: t })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ChooseTarget { .. }
                        | Action::Sacrifice { .. }
                        | Action::ChooseSearch { .. }
                        | Action::FinishSearch
                        | Action::FinishTargets
                        | Action::Pass
                )
            })
        })
        .unwrap_or_else(|| panic!("no continuation {:?} {actions:?}", g.state.pending))
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, t: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, t);
    }
    panic!("unsettled");
}
fn stack(g: &mut Game, id: CardId, pay: bool, t: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    assert!(matches!(g.state.pending, Pending::Kicker { .. }));
    assert!(g.observe(g.actor().unwrap()).payment_request.is_none());
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .cast
            .unwrap()
            .kicker,
        None
    );
    replay(g, Action::ChooseKicker { pay });
    while g.state.announcing.is_some() {
        choose(g, t);
    }
    let item = g.state.stack.last().unwrap();
    assert_eq!(item.cast.unwrap().kicker, Some(pay));
    assert_eq!(item.cast.unwrap().offspring, None);
}
#[test]
fn ten_complete_bodies_both_branches_seats_and_single_effects() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            for pay in [false, true] {
                let mut g = setup(p);
                let target = named(
                    &mut g,
                    if i == 6 || i == 7 || i == 8 {
                        p
                    } else {
                        p.other()
                    },
                    "Bear Cub",
                    if i == 6 || i == 7 || i == 8 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    },
                );
                let source = named(&mut g, p, n, Zone::Hand);
                let t = if i == 0 || i == 2 {
                    Some(Target::Player(p.other()))
                } else {
                    Some(Target::Card(target))
                };
                let life = g.state.players[p.other().index()].life;
                stack(&mut g, source, pay, t);
                settle(&mut g, t);
                match i {
                    0 => assert_eq!(
                        g.state.players[p.other().index()].life,
                        life - if pay { 4 } else { 2 }
                    ),
                    1 => {
                        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
                    }
                    2 => assert_eq!(
                        g.state.objects[target as usize].zone,
                        if pay {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    ),
                    3 => {}
                    4 | 5 => {
                        assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
                        assert_eq!(
                            g.state.players[p.index()].cards_drawn_this_turn,
                            u64::from(pay)
                        );
                    }
                    6 | 7 => {
                        assert_eq!(
                            g.state.objects[target as usize].zone,
                            if pay {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                        assert!(g.keywords(source).contains(Keywords::LIFELINK));
                        if i == 6 {
                            assert!(g.keywords(source).contains(Keywords::MENACE));
                        }
                    }
                    8 => {
                        assert_eq!(
                            g.state.objects[target as usize].zone,
                            if pay { Zone::Battlefield } else { Zone::Hand }
                        );
                        assert_eq!(g.state.objects[target as usize].tapped, pay);
                    }
                    9 => {
                        assert_eq!(
                            g.state.objects[source as usize].counters.plus_one,
                            if pay { 2 } else { 0 }
                        );
                        assert_eq!(g.keywords(source).contains(Keywords::TRAMPLE), pay);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
#[test]
fn choice_precedes_targets_and_typed_resources_and_rejects_actor_and_shortcuts() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
        let id = named(&mut g, Player::P0, n, Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Cast{card,..}|Action::BeginCast{card,..} if *card==id))
        );
        replay(&mut g, Action::AnnounceCast { card: id });
        let o = g.observe(Player::P0);
        assert_eq!(o.decision, "kicker");
        assert!(o.payment_request.is_none());
        assert!(o.announcing.unwrap().targets.is_empty());
        assert!(g.observe(Player::P1).actions.is_empty());
        let s = g.snapshot();
        assert_eq!(
            g.apply(Player::P1, Action::ChooseKicker { pay: false }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), s);
        reject(&mut g, Action::ChooseOffspring { pay: true });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
    }
}
#[test]
fn combined_fixed_costs_preserve_colors_and_printed_value_before_generic_reduction() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            let mut g = setup(p);
            named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let id = named(&mut g, p, n, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            replay(&mut g, Action::ChooseKicker { pay: true });
            let request = g.observe(p).payment_request.unwrap();
            let c = Card::from_name(n).unwrap();
            let (mut colored, base) = c.cost();
            let extra = c.kicker_cost().unwrap();
            for (a, b) in colored.iter_mut().zip(extra.colored) {
                *a += b;
            }
            assert_eq!(request.required, colored);
            assert_eq!(request.generic, base + extra.generic);
            if let Some(locked) = g.state.announcing.as_ref().unwrap().total_cost {
                assert_eq!(
                    locked,
                    ManaCost {
                        colored,
                        generic: base + extra.generic
                    }
                );
            }
        }
        let mut g = setup(p);
        for _ in 0..4 {
            named(&mut g, p, "Mocking Sprite", Zone::Battlefield);
        }
        let id = named(&mut g, p, "Burst Lightning", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        stack(&mut g, id, true, Some(Target::Player(p.other())));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g, None);
    }
}
#[test]
fn colored_kicker_funding_is_joint_and_cannot_spend_base_mana_twice() {
    for (n, pool) in [
        ("Gatekeeper of Malakir", [0, 0, 2, 0, 0, 0]),
        ("Into the Roil", [0, 1, 0, 0, 0, 3]),
        ("Gnarlid Colony", [0, 0, 0, 0, 1, 4]),
    ] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, Player::P0, n, Zone::Hand);
        g.state.players[0].mana = pool;
        replay(&mut g, Action::AnnounceCast { card: id });
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|a| !matches!(a, Action::Concede))
                .collect::<Vec<_>>(),
            vec![Action::ChooseKicker { pay: false }]
        );
        reject(&mut g, Action::ChooseKicker { pay: true });
    }
}
#[test]
fn kicked_entry_triggers_survive_source_departure_and_do_not_recur_on_new_entry() {
    for n in [
        "Gatekeeper of Malakir",
        "Nullpriest of Oblivion",
        "Sun-Blessed Healer",
    ] {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let grave = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, n, Zone::Hand);
            let t = if n == "Gatekeeper of Malakir" {
                Target::Player(p.other())
            } else {
                Target::Card(grave)
            };
            stack(&mut g, id, true, None);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            while g.state.announcing.is_some() {
                choose(&mut g, Some(t));
            }
            assert_eq!(g.state.stack.len(), 1);
            let old = g.object_ref(id);
            assert_eq!(g.state.stack[0].object, old);
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            assert_ne!(g.object_ref(id), old);
            settle(&mut g, Some(t));
            assert_eq!(
                g.state.objects[if n == "Gatekeeper of Malakir" {
                    foe
                } else {
                    grave
                } as usize]
                    .zone,
                if n == "Gatekeeper of Malakir" {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert!(g.state.stack.is_empty());
            assert!(g.state.pending_triggers.is_empty());
        }
    }
}
#[test]
fn unkicked_no_target_etbs_and_suppression_never_block_gnarlid_replacement() {
    for orb in ["Torpor Orb", "Doorkeeper Thrull"] {
        for n in [
            "Gatekeeper of Malakir",
            "Nullpriest of Oblivion",
            "Sun-Blessed Healer",
            "Gnarlid Colony",
        ] {
            let mut g = setup(Player::P0);
            named(&mut g, Player::P1, orb, Zone::Battlefield);
            let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
            let grave = named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
            let id = named(&mut g, Player::P0, n, Zone::Hand);
            stack(&mut g, id, true, None);
            settle(&mut g, None);
            assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
            assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.objects[id as usize].counters.plus_one,
                if n == "Gnarlid Colony" { 2 } else { 0 }
            );
        }
    }
    for n in [
        "Gatekeeper of Malakir",
        "Nullpriest of Oblivion",
        "Sun-Blessed Healer",
    ] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, n, Zone::Hand);
        stack(&mut g, id, false, None);
        settle(&mut g, None);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn gnarlid_grants_live_only_to_controlled_creatures_with_actual_plus_counters() {
    let mut g = setup(Player::P0);
    let yes = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let no = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let foe = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    for id in [yes, foe, land] {
        g.state.objects[id as usize].counters.plus_one = 1;
    }
    g.state.objects[no as usize].growth = Stats {
        power: 1,
        toughness: 1,
    };
    let colony = named(&mut g, Player::P0, "Gnarlid Colony", Zone::Hand);
    stack(&mut g, colony, true, None);
    settle(&mut g, None);
    for id in [yes, colony] {
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
    }
    for id in [no, foe, land] {
        assert!(!g.keywords(id).contains(Keywords::TRAMPLE));
    }
    g.state.objects[yes as usize].counters.plus_one = 0;
    assert!(!g.keywords(yes).contains(Keywords::TRAMPLE));
    g.move_object(colony, Zone::Hand);
    assert!(!g.keywords(colony).contains(Keywords::TRAMPLE));
    g.move_object(colony, Zone::Battlefield);
    assert_eq!(g.state.objects[colony as usize].counters.plus_one, 0);
    assert!(!g.keywords(colony).contains(Keywords::TRAMPLE));
}
#[test]
fn kicked_bounce_fizzles_without_draw_on_reincarnated_target() {
    for n in ["Into the Roil", "Whoosh!"] {
        let mut g = setup(Player::P0);
        let t = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, Player::P0, n, Zone::Hand);
        stack(&mut g, id, true, Some(Target::Card(t)));
        g.move_object(t, Zone::Exile);
        g.move_object(t, Zone::Battlefield);
        settle(&mut g, None);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert_eq!(g.state.objects[t as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn healer_exact_nonland_permanent_mana_value_filter_and_target_reincarnation() {
    let mut g = setup(Player::P0);
    let good = named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let artifact = named(&mut g, Player::P0, "Candlestick", Zone::Graveyard);
    let spell = named(&mut g, Player::P0, "Shock", Zone::Graveyard);
    let land = named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    let expensive = named(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let foreign = named(&mut g, Player::P1, "Bear Cub", Zone::Graveyard);
    let healer = named(&mut g, Player::P0, "Sun-Blessed Healer", Zone::Hand);
    stack(&mut g, healer, true, None);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    for id in [spell, land, expensive, foreign] {
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(id)
        }));
    }
    for id in [good, artifact] {
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(id)
        }));
    }
    choose(&mut g, Some(Target::Card(good)));
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    g.move_object(good, Zone::Hand);
    g.move_object(good, Zone::Graveyard);
    settle(&mut g, None);
    assert_eq!(g.state.objects[good as usize].zone, Zone::Graveyard);
}
#[test]
fn grow_single_search_full_private_commit_normal_entry_and_legal_fail_to_find() {
    for pay in [false, true] {
        for amount in 0..=if pay { 2 } else { 1 } {
            let mut g = setup(Player::P0);
            let id = named(&mut g, Player::P0, "Grow from the Ashes", Zone::Hand);
            stack(&mut g, id, pay, None);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            let o = g.observe(Player::P0);
            let search = o.search_choice.unwrap();
            assert_eq!(search.maximum, if pay { 2 } else { 1 });
            assert!(!search.tapped);
            assert_eq!(search.destination, crate::SearchDestination::Battlefield);
            assert!(g.observe(Player::P1).search_choice.is_none());
            assert!(g.observe(Player::P1).actions.is_empty());
            let candidates = search
                .cards
                .iter()
                .take(amount)
                .map(|v| v.id)
                .collect::<Vec<_>>();
            let original = g.state.players[0].library.len();
            for id in &candidates {
                replay(&mut g, Action::ChooseSearch { card: *id });
                assert_eq!(g.state.objects[*id as usize].zone, Zone::Library);
                assert_eq!(g.state.players[0].library.len(), original);
            }
            replay(&mut g, Action::FinishSearch);
            settle(&mut g, None);
            assert_eq!(g.state.players[0].library.len(), original - amount);
            for id in candidates {
                assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
                assert!(!g.state.objects[id as usize].tapped);
            }
        }
    }
}
#[test]
fn early_and_late_capacity_failures_restore_every_owned_choice_and_effect() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Gnarlid Colony", Zone::Hand);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(Player::P0, Action::AnnounceCast { card: id }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Gatekeeper of Malakir", Zone::Hand);
    stack(&mut g, id, true, None);
    replay(&mut g, Action::Pass);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    let r = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), r);
    let mut g = setup(Player::P0);
    let t = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Into the Roil", Zone::Hand);
    stack(&mut g, id, true, Some(Target::Card(t)));
    replay(&mut g, Action::Pass);
    g.state.players[0].cards_drawn_this_turn = u64::MAX;
    let saved = g.snapshot();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.state.objects[t as usize].zone, Zone::Battlefield);
}
#[test]
fn actual_exile_cast_permission_retains_kicker_and_forbids_other_origins() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Burst Lightning", Zone::Library);
            g.exile_top_play(p, 1, true);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
            replay(&mut g, Action::AnnounceExile { card: id });
            replay(&mut g, Action::ChooseKicker { pay });
            while g.state.announcing.is_some() {
                choose(&mut g, Some(Target::Player(p.other())));
            }
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.origin, Zone::Exile);
            assert_eq!(cast.method, CastMethod::Normal);
            assert_eq!(cast.kicker, Some(pay));
            assert_eq!(cast.offspring, None);
            assert!(g.state.exile_permissions.is_empty());
            settle(&mut g, None);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
        for z in [Zone::Graveyard, Zone::Exile, Zone::Library] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Burst Lightning", z);
            for a in [
                Action::AnnounceCast { card: id },
                Action::AnnounceExile { card: id },
                Action::AnnounceFlashback { card: id },
            ] {
                reject(&mut g, a);
            }
        }
    }
}
#[test]
fn paid_but_countered_spell_never_enters_and_reanimated_gnarlid_is_unkicked() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Gnarlid Colony", Zone::Hand);
    stack(&mut g, id, true, None);
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: counter });
    while g.state.announcing.is_some() {
        choose(&mut g, Some(Target::Spell(id)));
    }
    settle(&mut g, None);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[id as usize].counters.plus_one, 0);
    g.state.priority = Player::P0;
    let zuko = named(&mut g, Player::P0, "Zuko's Conviction", Zone::Hand);
    stack(&mut g, zuko, true, Some(Target::Card(id)));
    settle(&mut g, None);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
    assert!(g.state.objects[id as usize].tapped);
    assert_eq!(g.state.objects[id as usize].counters.plus_one, 0);
}
#[test]
fn damage_replacement_and_gnarlid_entry_observers_use_the_selected_branch() {
    for pay in [false, true] {
        let mut g = setup(Player::P0);
        let target = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let id = named(&mut g, Player::P0, "Firebending Lesson", Zone::Hand);
        stack(&mut g, id, pay, Some(Target::Card(target)));
        settle(&mut g, None);
        assert_eq!(
            g.state.objects[target as usize].zone,
            if pay {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        if !pay {
            assert_eq!(g.state.objects[target as usize].damage, 2);
        }
    }
    for pay in [false, true] {
        let mut g = setup(Player::P0);
        let clown = named(&mut g, Player::P0, "Vicious Clown", Zone::Battlefield);
        let id = named(&mut g, Player::P0, "Gnarlid Colony", Zone::Hand);
        stack(&mut g, id, pay, None);
        settle(&mut g, None);
        assert_eq!(g.stats(id).power, if pay { 4 } else { 2 });
        assert_eq!(g.stats(clown).power, if pay { 2 } else { 4 });
    }
}
#[test]
fn total_kicker_cost_locks_before_mana_abilities_even_when_reducers_depart() {
    let mut g = setup(Player::P0);
    g.state.players[0].mana = [0; 6];
    let reducers = (0..4)
        .map(|_| named(&mut g, Player::P0, "Mocking Sprite", Zone::Battlefield))
        .collect::<Vec<_>>();
    let mountain = named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Burst Lightning", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseKicker { pay: true });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(
        g.state.announcing.as_ref().unwrap().total_cost,
        Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0
        })
    );
    for source in reducers {
        g.move_object(source, Zone::Hand);
    }
    replay(&mut g, Action::TapMana { card: mountain });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 0);
    choose(&mut g, None);
    settle(&mut g, None);
    assert_eq!(g.state.players[1].life, 16);
}
#[test]
fn multi_land_entry_capacity_failure_restores_full_private_search_and_rng() {
    let mut g = setup(Player::P0);
    named(
        &mut g,
        Player::P0,
        "Iridescent Vinelasher",
        Zone::Battlefield,
    );
    let source = named(&mut g, Player::P0, "Grow from the Ashes", Zone::Hand);
    stack(&mut g, source, true, None);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    let cards = g
        .observe(Player::P0)
        .search_choice
        .unwrap()
        .cards
        .into_iter()
        .take(2)
        .map(|v| v.id)
        .collect::<Vec<_>>();
    for id in &cards {
        replay(&mut g, Action::ChooseSearch { card: *id });
    }
    g.state.next_stack_id = u64::MAX - 1;
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::FinishSearch),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.observe(Player::P0).search_choice.unwrap().selected, cards);
    for id in cards {
        assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
    }
}
