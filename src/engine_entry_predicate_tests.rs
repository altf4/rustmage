use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "HYDRA Assault Robot",
    "Impact Tremors",
    "Marketwatch Phantom",
    "Neighborhood Guardian",
    "Snarling Gorehound",
    "Valley Mightcaller",
    "Vial Smasher, Gleeful Grenadier",
    "Vicious Clown",
    "Warleader's Call",
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
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn choose(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. }
                    | Action::PayCosts { .. }
                    | Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::Discard { .. }
                    | Action::ChooseTarget { .. }
                    | Action::FinishTargets
                    | Action::ArrangeLibrary { .. }
            )
        })
        .expect("fixture choice");
    replay(g, a);
}
fn settle(g: &mut Game) -> usize {
    let mut private = 0;
    g.prepare_triggers();
    for _ in 0..160 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return private;
        }
        private += usize::from(g.observe(g.actor().unwrap()).library_choice.is_some());
        choose(g);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId) -> usize {
    replay(g, Action::AnnounceCast { card: id });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(g);
    }
    settle(g)
}
fn enter(g: &mut Game, ids: &[CardId]) {
    let events = g.move_objects_raw(ids, Zone::Battlefield);
    g.finish_zone_events(&events, &[]);
}
#[test]
fn all_nine_complete_cards_cast_and_resolve_their_entry_effects_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source);
            let before = g.object_view(source);
            let mate = named(
                &mut g,
                p,
                match i {
                    0 => "Three Tree Mascot",
                    5 => "Gigantoad",
                    6 => "Swab Goblin",
                    _ => "Bear Cub",
                },
                Zone::Hand,
            );
            let private = cast(&mut g, mate);
            let after = g.object_view(source);
            match i {
                0 | 1 | 6 | 8 => assert_eq!(g.state.players[p.other().index()].life, 19),
                2 => assert_eq!(after.keywords, Keywords::FLYING),
                3 => {
                    let total = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.zone == Zone::Battlefield && o.owner == p)
                        .map(|(id, _)| g.object_view(id as CardId).power)
                        .sum::<i128>();
                    assert_eq!(total, 5);
                }
                4 => assert_eq!(private, 1),
                5 => assert_eq!(after.counters.plus_one, 1),
                7 => assert_eq!(
                    (after.power, after.toughness),
                    (before.power + 2, before.toughness)
                ),
                _ => unreachable!(),
            }
            if i == 8 {
                let v = g.object_view(mate);
                assert_eq!((v.power, v.toughness), (3, 3));
            }
        }
    }
}
#[test]
fn entry_time_power_uses_final_continuous_effects_and_entry_counters_in_both_orders() {
    for reverse in [false, true] {
        let mut g = setup(Player::P0);
        for name in [NAMES[2], NAMES[3], NAMES[4], NAMES[7]] {
            named(&mut g, Player::P0, name, Zone::Battlefield);
        }
        let call = named(&mut g, Player::P0, NAMES[8], Zone::Hand);
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        let mut batch = vec![call, bear];
        if reverse {
            batch.reverse();
        }
        enter(&mut g, &batch);
        assert_eq!(g.object_view(bear).power, 3);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].object.card, call);
        g.state.pending_triggers.clear();
        let stone = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Hand);
        enter(&mut g, &[stone]);
        assert_eq!(g.object_view(stone).power, 3);
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.state.pending_triggers.clear();
        let events = g.move_objects_raw(&[call], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        let stone2 = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Hand);
        enter(&mut g, &[stone2]);
        assert_eq!(g.object_view(stone2).power, 2);
        assert_eq!(g.state.pending_triggers.len(), 4);
    }
}
#[test]
fn power_is_tested_at_entry_and_not_rechecked_on_resolution() {
    let mut g = setup(Player::P0);
    let clown = named(&mut g, Player::P0, NAMES[7], Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    assert_eq!(g.state.pending_triggers.len(), 1);
    g.state.objects[bear as usize].growth.power = 20;
    settle(&mut g);
    assert_eq!(g.object_view(clown).power, 4);
}
#[test]
fn power_predicate_includes_negative_power_and_excludes_self_enemy_and_noncreatures() {
    for name in [NAMES[2], NAMES[3], NAMES[4], NAMES[7]] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        enter(&mut g, &[source]);
        assert!(g.state.pending_triggers.is_empty());
        let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
        enter(&mut g, &[enemy]);
        assert!(g.state.pending_triggers.is_empty());
        let artifact = named(&mut g, Player::P0, "Torpor Orb", Zone::Hand);
        enter(&mut g, &[artifact]);
        assert!(g.state.pending_triggers.is_empty());
        let events = g.move_objects_raw(&[artifact], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        let events = g.move_objects_raw(&[bear], Zone::Battlefield);
        // Test the predicate boundary with a negative post-event characteristic.
        g.state.objects[bear as usize].growth.power = -10;
        g.finish_zone_events(&events, &[]);
        assert_eq!(g.object_view(bear).power, -8);
        assert_eq!(g.state.pending_triggers.len(), 1);
    }
}
#[test]
fn subtype_and_kind_unions_count_once_per_other_qualifying_permanent() {
    let mut g = setup(Player::P0);
    let hydra = named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    let both = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    enter(&mut g, &[both]);
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.state.pending_triggers[0].object.card, hydra);
    g.state.pending_triggers.clear();
    for name in ["Three Tree Mascot", "Flying Octobot"] {
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        enter(&mut g, &[id]);
        assert_eq!(g.state.pending_triggers.len(), 2);
        g.state.pending_triggers.clear();
    }
    let enemy = named(&mut g, Player::P1, NAMES[0], Zone::Hand);
    enter(&mut g, &[enemy]);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn named_subtype_members_and_outlaws_match_singular_events() {
    // The generated union itself has no creature-only predicate: CR 109.2/700.12.
    let mut g = setup(Player::P0);
    let valley = named(&mut g, Player::P0, NAMES[5], Zone::Battlefield);
    for name in ["Gigantoad", "Hinterland Sanctifier"] {
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        enter(&mut g, &[id]);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.object.card == valley)
                .count(),
            1
        );
        g.state.pending_triggers.clear();
    }
    let vial = named(&mut g, Player::P0, NAMES[6], Zone::Battlefield);
    for name in ["Swab Goblin", "Outlaw Medic", "Burrog Banemaker"] {
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        enter(&mut g, &[id]);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|t| t.object.card == vial)
                .count(),
            1
        );
        g.state.pending_triggers.clear();
    }
    let innocent = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[innocent]);
    assert!(
        !g.state
            .pending_triggers
            .iter()
            .any(|t| t.object.card == vial)
    );
}
#[test]
fn simultaneous_tokens_have_one_trigger_each_and_prohibitions_prevent_all_new_conditions() {
    let rabbit = Card::catalogue()
        .find(|c| {
            c.token_info().is_some_and(|t| t.subtypes == "Rabbit")
                && matches!(c.effect(), Effect::Creature(1, 1))
        })
        .unwrap();
    for blocked in [false, true] {
        let mut g = setup(Player::P0);
        for name in NAMES {
            named(&mut g, Player::P0, name, Zone::Battlefield);
        }
        if blocked {
            named(&mut g, Player::P1, "Torpor Orb", Zone::Battlefield);
        }
        // Warleader makes a 1/1 Rabbit 2/2: all four small-creature observers match.
        g.create_tokens(Player::P0, rabbit, 2, false);
        assert_eq!(g.state.pending_triggers.len(), if blocked { 0 } else { 14 });
    }
}
#[test]
fn source_departure_and_reentry_do_not_grant_old_temporary_or_counter_effects_to_new_incarnation() {
    for name in [NAMES[2], NAMES[5], NAMES[7]] {
        let mut g = setup(Player::P0);
        let s = named(&mut g, Player::P0, name, Zone::Battlefield);
        let mate = named(
            &mut g,
            Player::P0,
            if name == NAMES[5] {
                "Gigantoad"
            } else {
                "Bear Cub"
            },
            Zone::Hand,
        );
        enter(&mut g, &[mate]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let old = g.object_ref(s);
        let events = g.move_objects_raw(&[s], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        enter(&mut g, &[s]);
        assert_ne!(g.object_ref(s), old);
        let before = g.object_view(s);
        settle(&mut g);
        let after = g.object_view(s);
        assert_eq!(
            (after.power, after.toughness, after.keywords, after.counters),
            (
                before.power,
                before.toughness,
                before.keywords,
                before.counters
            )
        );
    }
}
#[test]
fn temporary_source_growth_and_flying_expire_but_valley_counters_remain() {
    let mut g = setup(Player::P0);
    let phantom = named(&mut g, Player::P0, NAMES[2], Zone::Battlefield);
    let clown = named(&mut g, Player::P0, NAMES[7], Zone::Battlefield);
    let valley = named(&mut g, Player::P0, NAMES[5], Zone::Battlefield);
    let frog = named(&mut g, Player::P0, "Hinterland Sanctifier", Zone::Hand);
    enter(&mut g, &[frog]);
    settle(&mut g);
    assert!(g.object_view(phantom).keywords.contains(Keywords::FLYING));
    assert_eq!(g.object_view(clown).power, 4);
    assert_eq!(g.object_view(valley).counters.plus_one, 1);
    g.finish_cleanup();
    assert!(!g.object_view(phantom).keywords.contains(Keywords::FLYING));
    assert_eq!(g.object_view(clown).power, 2);
    assert_eq!(g.object_view(valley).counters.plus_one, 1);
}
#[test]
fn guardian_targets_remain_legal_and_fizzle_after_source_target_reentry() {
    let mut g = setup(Player::P0);
    let guardian = named(&mut g, Player::P0, NAMES[3], Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[mate]);
    g.prepare_triggers();
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert!(
        g.apply(
            g.actor().unwrap(),
            Action::ChooseTarget {
                target: Target::Card(enemy)
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(guardian),
        },
    );
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(&mut g);
    }
    let old = g.object_ref(guardian);
    let events = g.move_objects_raw(&[guardian], Zone::Exile);
    g.finish_zone_events(&events, &[]);
    enter(&mut g, &[guardian]);
    assert_ne!(g.object_ref(guardian), old);
    settle(&mut g);
    assert_eq!(g.object_view(guardian).power, 2);
}
#[test]
fn small_entry_surveil_is_private_and_snapshot_owned() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[4], Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[mate]);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert!(g.observe(Player::P0).library_choice.is_some());
    assert!(g.observe(Player::P1).library_choice.is_none());
    let snapshot = g.snapshot();
    assert!(g.apply(Player::P1, Action::Pass).is_err());
    assert_eq!(g.snapshot(), snapshot);
    choose(&mut g);
    assert!(g.observe(Player::P0).library_choice.is_none());
}
#[test]
fn union_counter_overflow_restores_resolution_and_revision() {
    let mut g = setup(Player::P0);
    let valley = named(&mut g, Player::P0, NAMES[5], Zone::Battlefield);
    g.state.objects[valley as usize].counters.plus_one = u64::MAX;
    let mate = named(&mut g, Player::P0, "Hinterland Sanctifier", Zone::Hand);
    enter(&mut g, &[mate]);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
