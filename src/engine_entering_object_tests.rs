use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 3] = [
    "Good-Fortune Unicorn",
    "Serra Redeemer",
    "Wartime Protestors",
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
fn three_whole_bodies_cast_and_grant_counters_and_haste_to_the_exact_entrant_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source);
            let mate = named(
                &mut g,
                p,
                if i == 2 {
                    "First-Time Flyer"
                } else {
                    "Bear Cub"
                },
                Zone::Hand,
            );
            cast(&mut g, mate);
            let v = g.object_view(mate);
            assert_eq!(v.counters.plus_one, if i == 1 { 2 } else { 1 });
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            assert_eq!(v.keywords.contains(Keywords::HASTE), i == 2);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            assert_eq!(
                g.legal_actions().contains(&Action::Attack { card: mate }),
                i == 2
            );
            assert_eq!(g.observe(p).schema, OBSERVATION_SCHEMA);
        }
    }
}
#[test]
fn entry_packets_are_explicit_non_targets_and_old_triggers_also_capture_the_entry() {
    let mut g = setup(Player::P0);
    let unicorn = named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    let hunter = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
    enter(&mut g, &[hunter]);
    assert_eq!(g.state.pending_triggers.len(), 2);
    for item in &g.state.pending_triggers {
        assert_eq!(item.entering_object, Some(g.object_ref(hunter)));
        assert!(item.targets.is_empty());
        assert!(!item.program().unwrap().0.has_targets());
    }
    assert!(
        g.state
            .pending_triggers
            .iter()
            .any(|t| t.object.card == unicorn)
    );
    settle(&mut g);
    assert_eq!(g.object_view(hunter).counters.plus_one, 1);
}
#[test]
fn an_observer_leaving_does_not_cancel_the_captured_entry_effect() {
    for name in NAMES {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let mate = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
        enter(&mut g, &[mate]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let events = g.move_objects_raw(&[source], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        settle(&mut g);
        assert_eq!(
            g.object_view(mate).counters.plus_one,
            if name == NAMES[1] { 2 } else { 1 }
        );
    }
}
#[test]
fn an_entrant_leaving_and_returning_cannot_receive_an_old_entry_effect() {
    for name in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, name, Zone::Battlefield);
        let mate = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
        enter(&mut g, &[mate]);
        let old = g.object_ref(mate);
        let events = g.move_objects_raw(&[mate], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        named(&mut g, Player::P1, "Torpor Orb", Zone::Battlefield);
        enter(&mut g, &[mate]);
        assert_ne!(g.object_ref(mate), old);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].entering_object, Some(old));
        settle(&mut g);
        assert_eq!(g.object_view(mate).counters.plus_one, 0);
        assert!(!g.object_view(mate).keywords.contains(Keywords::HASTE));
    }
}
#[test]
fn recycled_token_slots_do_not_alias_a_captured_entrant() {
    let rabbit = Card::catalogue()
        .find(|c| {
            c.token_info().is_some_and(|t| t.subtypes == "Rabbit")
                && matches!(c.effect(), Effect::Creature(1, 1))
        })
        .unwrap();
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    g.create_tokens(Player::P0, rabbit, 1, false);
    let old = g.state.pending_triggers[0].entering_object.unwrap();
    g.move_object(old.card, Zone::Graveyard);
    g.state_based_actions();
    named(&mut g, Player::P1, "Torpor Orb", Zone::Battlefield);
    g.create_tokens(Player::P0, rabbit, 1, false);
    assert_ne!(g.object_ref(old.card), old);
    settle(&mut g);
    assert_eq!(g.object_view(old.card).counters.plus_one, 0);
}
#[test]
fn simultaneous_sources_and_entrants_capture_distinct_refs_and_resolve_all_matching_effects() {
    let mut g = setup(Player::P0);
    let good = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    let serra = named(&mut g, Player::P0, NAMES[1], Zone::Hand);
    let war = named(&mut g, Player::P0, NAMES[2], Zone::Hand);
    let flyer = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
    enter(&mut g, &[flyer, war, serra, good]);
    assert_eq!(g.state.pending_triggers.len(), 6);
    for item in &g.state.pending_triggers {
        assert!(item.entering_object.is_some());
        assert!(item.targets.is_empty());
    }
    settle(&mut g);
    assert_eq!(g.object_view(good).counters.plus_one, 2);
    assert_eq!(g.object_view(serra).counters.plus_one, 1);
    assert_eq!(g.object_view(war).counters.plus_one, 1);
    assert_eq!(g.object_view(flyer).counters.plus_one, 4);
    assert!(g.object_view(flyer).keywords.contains(Keywords::HASTE));
}
#[test]
fn two_redeemers_use_entry_time_power_and_both_counter_effects_resolve() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[1], Zone::Battlefield);
    named(&mut g, Player::P0, NAMES[1], Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    assert_eq!(g.state.pending_triggers.len(), 2);
    settle(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 4);
    assert_eq!(g.object_view(bear).power, 6);
}
#[test]
fn redeemer_sees_continuous_stats_at_entry_while_unicorn_ignores_power() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    named(&mut g, Player::P0, NAMES[1], Zone::Battlefield);
    named(&mut g, Player::P0, "Warleader's Call", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    assert_eq!(g.state.pending_triggers.len(), 2);
    settle(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert_eq!(g.object_view(bear).power, 4);
}
#[test]
fn opponents_and_non_ally_entrants_are_excluded_by_their_original_conditions() {
    for (i, name) in NAMES.iter().enumerate() {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, name, Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, "First-Time Flyer", Zone::Hand);
        enter(&mut g, &[enemy]);
        assert!(g.state.pending_triggers.is_empty());
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        enter(&mut g, &[bear]);
        assert_eq!(g.state.pending_triggers.len(), usize::from(i != 2));
    }
}
#[test]
fn entry_suppression_blocks_reference_capture_and_haste_expires_without_removing_counters() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[2], Zone::Battlefield);
    let flyer = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
    cast(&mut g, flyer);
    assert!(g.object_view(flyer).keywords.contains(Keywords::HASTE));
    g.finish_cleanup();
    assert!(!g.object_view(flyer).keywords.contains(Keywords::HASTE));
    assert_eq!(g.object_view(flyer).counters.plus_one, 1);
    named(&mut g, Player::P1, "Doorkeeper Thrull", Zone::Battlefield);
    let next = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
    enter(&mut g, &[next]);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.object_view(next).counters.plus_one, 0);
}
#[test]
fn captured_entry_waits_safely_on_stack_through_an_owned_private_surveil_frame() {
    let mut g = setup(Player::P0);
    let good = named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    named(&mut g, Player::P0, "Snarling Gorehound", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    g.prepare_triggers();
    for _ in 0..30 {
        if g.observe(Player::P0).library_choice.is_some() {
            break;
        }
        choose(&mut g);
    }
    assert!(g.observe(Player::P0).library_choice.is_some());
    assert!(g.observe(Player::P1).library_choice.is_none());
    assert!(
        g.state
            .stack
            .iter()
            .any(|s| s.object.card == good && s.entering_object == Some(g.object_ref(bear)))
    );
    settle(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
}
#[test]
fn entering_counter_overflow_is_atomic_and_does_not_grant_haste() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[2], Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "First-Time Flyer", Zone::Hand);
    enter(&mut g, &[mate]);
    g.state.objects[mate as usize].counters.plus_one = u64::MAX;
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
    assert!(!g.object_view(mate).keywords.contains(Keywords::HASTE));
}
#[test]
fn ordinary_spell_activation_and_death_packets_have_no_entering_reference() {
    let mut g = setup(Player::P0);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: bear });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(&mut g);
    }
    assert_eq!(g.state.stack.last().unwrap().entering_object, None);
    settle(&mut g);
    let elder = named(&mut g, Player::P0, "Warren Elder", Zone::Battlefield);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: elder,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(&mut g);
    }
    assert_eq!(g.state.stack.last().unwrap().entering_object, None);
    settle(&mut g);
    let inspector = named(&mut g, Player::P0, "Callous Inspector", Zone::Battlefield);
    g.move_object(inspector, Zone::Graveyard);
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.state.pending_triggers[0].entering_object, None);
}
#[test]
fn an_entry_effect_cannot_be_redirected_by_submitting_a_target_action() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[mate]);
    g.prepare_triggers();
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert!(
        g.apply(
            g.actor().unwrap(),
            Action::ChooseTarget {
                target: Target::Card(mate)
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    assert!(g.state.stack.last().unwrap().targets.is_empty());
    settle(&mut g);
    assert_eq!(g.object_view(mate).counters.plus_one, 1);
}
#[test]
fn captured_counter_placement_annihilates_entry_minus_counters_in_the_normal_sba_batch() {
    for (source, remaining, power) in [(NAMES[0], 1, 3), (NAMES[1], 0, 4)] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, source, Zone::Battlefield);
        let entrant = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Hand);
        cast(&mut g, entrant);
        let v = g.object_view(entrant);
        assert_eq!(v.counters.plus_one, 0);
        assert_eq!(v.counters.minus_one, remaining);
        assert_eq!((v.power, v.toughness), (power, power));
    }
}
