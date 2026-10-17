use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "Aunt May",
    "Dreamdew Entrancer",
    "Fear of Immobility",
    "Hazardroot Herbalist",
    "Ingenious Leonin",
    "Little Bear",
    "Thwip!",
    "Wakandan Royal Guard",
    "Yip Yip!",
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
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
fn choose(g: &mut Game, target: Option<CardId>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|id| {
            actions.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::Pass
                        | Action::PayResolutionMana { .. }
                )
            })
        })
        .or_else(|| {
            actions
                .iter()
                .find(|a| matches!(a, Action::ChooseTarget { .. }))
        })
        .expect("continuation")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.outcome().is_some()
            || g.state.announcing.is_none()
                && g.state.resolving.is_none()
                && g.state.stack.is_empty()
                && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn cast_stack(g: &mut Game, source: CardId, target: Option<CardId>) {
    replay(g, Action::AnnounceCast { card: source });
    while g.state.announcing.is_some() {
        choose(g, target);
    }
}
fn trigger_stack(g: &mut Game, source: CardId, target: Option<CardId>) {
    let view = g.object_view(source);
    assert!(g.queue_trigger(&view, 0));
    g.prepare_triggers();
    while g.state.pending != Pending::Priority {
        choose(g, target);
    }
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        replay(g, Action::Attack { card: id });
    }
    replay(g, Action::FinishAttackers);
}
#[test]
fn all_nine_full_bodies_cast_and_execute_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let target = named(
                &mut g,
                if i == 2 { p.other() } else { p },
                "Three Tree Mascot",
                Zone::Battlefield,
            );
            g.state.objects[target as usize].tapped = i == 5;
            let source = named(&mut g, p, name, Zone::Hand);
            cast_stack(&mut g, source, Some(target));
            settle(&mut g, Some(target));
            match i {
                0 => {
                    let entrant = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
                    cast_stack(&mut g, entrant, None);
                    settle(&mut g, None);
                    assert_eq!(g.state.players[p.index()].life, 21);
                    assert_eq!(g.object_view(entrant).counters.plus_one, 1);
                }
                1 => {
                    let v = g.object_view(target);
                    assert!(v.tapped);
                    assert_eq!(v.counters.stun, 3);
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
                }
                2 => {
                    assert!(g.object_view(target).tapped);
                    assert_eq!(g.object_view(target).counters.stun, 1);
                }
                3 => {
                    let token = Card::catalogue()
                        .find(|c| c.is_token() && c.is_creature())
                        .unwrap();
                    let token = spawn(&mut g, p, token, Zone::Battlefield);
                    let old_power = g.object_view(token).power;
                    attack(&mut g, p, &[target]);
                    settle(&mut g, Some(token));
                    assert_eq!(g.object_view(token).power, old_power + 1);
                    assert!(g.keywords(token).contains(Keywords::DEATHTOUCH));
                }
                4 => {
                    attack(&mut g, p, &[target]);
                    settle(&mut g, None);
                    g.state.players[p.index()].mana = [30; 6];
                    replay(
                        &mut g,
                        Action::AnnounceActivate {
                            card: source,
                            ability: 0,
                            counter_payment: Counters::default(),
                        },
                    );
                    while g.state.announcing.is_some() {
                        choose(&mut g, Some(target));
                    }
                    settle(&mut g, Some(target));
                    assert_eq!(g.object_view(target).counters.plus_one, 1);
                    assert!(g.keywords(target).contains(Keywords::FIRST_STRIKE));
                }
                5 => {
                    assert!(!g.object_view(target).tapped);
                    assert_eq!(g.object_view(target).counters.plus_one, 1);
                }
                6 => {
                    assert!(g.keywords(target).contains(Keywords::FLYING));
                    assert_eq!(g.object_view(target).power, 4);
                    assert_eq!(g.state.players[p.index()].life, 22);
                }
                7 => assert_eq!(g.object_view(target).counters.plus_one, 2),
                8 => {
                    assert!(g.keywords(target).contains(Keywords::FLYING));
                    assert_eq!(g.object_view(target).power, 4);
                }
                _ => unreachable!(),
            }
            assert_eq!(g.observe(p).schema, OBSERVATION_SCHEMA);
        }
    }
}
#[test]
fn optional_etbs_distinguish_both_relations_and_zero_targets() {
    for p in [Player::P0, Player::P1] {
        for name in ["Dreamdew Entrancer", "Fear of Immobility"] {
            for relation in [None, Some(p), Some(p.other())] {
                let mut g = setup(p);
                let target = relation.map(|q| named(&mut g, q, "Bear Cub", Zone::Battlefield));
                let source = named(&mut g, p, name, Zone::Battlefield);
                trigger_stack(&mut g, source, target);
                let chosen = g.state.stack.last().unwrap();
                assert_eq!(chosen.targets.len(), usize::from(relation.is_some()));
                settle(&mut g, target);
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    if name == "Dreamdew Entrancer" && relation == Some(p) {
                        2
                    } else {
                        0
                    }
                );
                if let Some(t) = target {
                    assert!(g.object_view(t).tapped);
                    assert_eq!(
                        g.object_view(t).counters.stun,
                        if name == "Dreamdew Entrancer" {
                            3
                        } else {
                            u64::from(relation == Some(p.other()))
                        }
                    );
                }
            }
        }
    }
}
#[test]
fn subtype_conditions_use_current_subject_and_changeling_not_source_or_owner() {
    for p in [Player::P0, Player::P1] {
        for (name, positive) in [
            ("Little Bear", "Bear Cub"),
            ("Thwip!", "Attercop"),
            ("Yip Yip!", "First-Time Flyer"),
            ("Wakandan Royal Guard", "Wakandan Royal Guard"),
        ] {
            for target_name in [positive, "Three Tree Mascot", "Llanowar Elves"] {
                let mut g = setup(p);
                let target = named(&mut g, p, target_name, Zone::Battlefield);
                g.state.objects[target as usize].tapped = true;
                let source = named(&mut g, p, name, Zone::Hand);
                cast_stack(&mut g, source, Some(target));
                settle(&mut g, Some(target));
                let yes = target_name != "Llanowar Elves";
                match name {
                    "Little Bear" => {
                        assert!(!g.object_view(target).tapped);
                        assert_eq!(g.object_view(target).counters.plus_one, u64::from(yes));
                    }
                    "Thwip!" => {
                        assert!(g.keywords(target).contains(Keywords::FLYING));
                        assert_eq!(
                            g.state.players[p.index()].life,
                            20 + if yes { 2 } else { 0 }
                        );
                    }
                    "Yip Yip!" => assert_eq!(g.keywords(target).contains(Keywords::FLYING), yes),
                    _ => assert_eq!(
                        g.object_view(target).counters.plus_one,
                        if yes { 2 } else { 1 }
                    ),
                }
            }
        }
    }
}
#[test]
fn wakandan_self_is_one_other_hero_is_two_and_the_compiler_uses_one_counter_operation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Wakandan Royal Guard", Zone::Battlefield);
        trigger_stack(&mut g, source, Some(source));
        settle(&mut g, Some(source));
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        let program = g.object_view(source).card.triggers()[0].program;
        assert!(matches!(
            program,
            [
                Instruction::IfSubject { skip: 2, .. },
                Instruction::PutCounters { amount: 2, .. },
                Instruction::Skip(1),
                Instruction::PutCounters { amount: 1, .. }
            ]
        ));
    }
}
#[test]
fn aunt_may_captures_each_entrant_and_life_is_independent_of_departed_source_or_entrant() {
    for p in [Player::P0, Player::P1] {
        for departed in [0, 1, 2] {
            let mut g = setup(p);
            let aunt = named(&mut g, p, "Aunt May", Zone::Battlefield);
            let spider = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
            g.move_object(spider, Zone::Battlefield);
            let old = g.object_ref(spider);
            assert_eq!(g.state.pending_triggers.len(), 1);
            assert_eq!(g.state.pending_triggers[0].entering_object, Some(old));
            assert!(g.state.pending_triggers[0].targets.is_empty());
            if departed == 1 {
                g.move_object(aunt, Zone::Graveyard);
            }
            if departed == 2 {
                g.move_object(spider, Zone::Graveyard);
                g.move_object_raw(spider, Zone::Battlefield);
            }
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(
                g.object_view(spider).counters.plus_one,
                if departed == 2 { 0 } else { 1 }
            );
        }
        let mut g = setup(p);
        let aunt = named(&mut g, p, "Aunt May", Zone::Hand);
        g.move_object(aunt, Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Attercop", Zone::Hand);
        g.move_object(foe, Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        assert!(g.state.pending_triggers.is_empty());
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.object_view(bear).counters.plus_one, 0);
    }
}
#[test]
fn aunt_may_multiple_entering_objects_keep_their_own_context_and_suppression_is_respected() {
    for p in [Player::P0, Player::P1] {
        for suppressed in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Aunt May", Zone::Battlefield);
            if suppressed {
                named(&mut g, p, "Torpor Orb", Zone::Battlefield);
            }
            let spider = named(&mut g, p, "Three Tree Mascot", Zone::Hand);
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            let events = g.move_objects_raw(&[spider, bear], Zone::Battlefield);
            g.finish_zone_events(&events, &[]);
            assert_eq!(
                g.state.pending_triggers.len(),
                if suppressed { 0 } else { 2 }
            );
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.index()].life,
                if suppressed { 20 } else { 22 }
            );
            assert_eq!(
                g.object_view(spider).counters.plus_one,
                u64::from(!suppressed)
            );
            assert_eq!(g.object_view(bear).counters.plus_one, 0);
        }
    }
}
#[test]
fn illegal_targets_fizzle_the_full_body_and_cannot_migrate_to_new_incarnations() {
    for name in [
        "Dreamdew Entrancer",
        "Fear of Immobility",
        "Little Bear",
        "Thwip!",
        "Wakandan Royal Guard",
        "Yip Yip!",
    ] {
        for loss in [0, 1, 2] {
            let mut g = setup(Player::P0);
            let target = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
            let source = named(
                &mut g,
                Player::P0,
                name,
                if name.ends_with('!') {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            if name.ends_with('!') {
                cast_stack(&mut g, source, Some(target));
            } else {
                trigger_stack(&mut g, source, Some(target));
            }
            match loss {
                0 => {
                    g.move_object(target, Zone::Graveyard);
                }
                1 => {
                    g.move_object(target, Zone::Graveyard);
                    g.move_object_raw(target, Zone::Battlefield);
                }
                _ => {
                    g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
                    g.state.objects[target as usize].controller = Player::P1;
                }
            }
            settle(&mut g, None);
            assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[0].life, 20);
            let v = g.object_view(target);
            assert_eq!(v.counters, Counters::default());
            assert_eq!(g.state.objects[target as usize].growth, Stats::default());
        }
    }
}
#[test]
fn hazardroot_uses_one_aggregate_attack_and_only_tokens_gain_deathtouch() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Hazardroot Herbalist", Zone::Battlefield);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
            let target = if token {
                spawn(
                    &mut g,
                    p,
                    Card::catalogue()
                        .find(|c| c.is_token() && c.is_creature())
                        .unwrap(),
                    Zone::Battlefield,
                )
            } else {
                b
            };
            let power = g.object_view(target).power;
            attack(&mut g, p, &[a, b]);
            assert_eq!(
                g.state.pending_triggers.len()
                    + g.state.ordering_triggers.len()
                    + g.state.stack.len()
                    + usize::from(g.state.announcing.is_some()),
                1
            );
            settle(&mut g, Some(target));
            assert_eq!(g.object_view(target).power, power + 1);
            assert_eq!(g.keywords(target).contains(Keywords::DEATHTOUCH), token);
        }
    }
}
#[test]
fn leonin_enforces_other_owned_attacking_subject_and_rechecks_combat_membership() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Ingenious Leonin", Zone::Battlefield);
        let cat = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let idle = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        attack(&mut g, p, &[source, cat]);
        settle(&mut g, None);
        g.state.players[p.index()].mana = [30; 6];
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for id in [source, idle, foe] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(cat),
            },
        );
        while g.state.announcing.is_some() {
            choose(&mut g, None);
        }
        g.state.attackers.retain(|a| a.card != cat);
        settle(&mut g, None);
        assert_eq!(g.object_view(cat).counters.plus_one, 0);
        assert!(!g.keywords(cat).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn independent_leonin_ability_survives_source_departure_but_noncat_has_no_first_strike() {
    for p in [Player::P0, Player::P1] {
        for n in ["Three Tree Mascot", "Bear Cub"] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Ingenious Leonin", Zone::Battlefield);
            let target = named(&mut g, p, n, Zone::Battlefield);
            attack(&mut g, p, &[target]);
            settle(&mut g, None);
            g.state.players[p.index()].mana = [30; 6];
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            while g.state.announcing.is_some() {
                choose(&mut g, Some(target));
            }
            g.move_object(source, Zone::Graveyard);
            settle(&mut g, Some(target));
            assert_eq!(g.object_view(target).counters.plus_one, 1);
            assert_eq!(
                g.keywords(target).contains(Keywords::FIRST_STRIKE),
                n == "Three Tree Mascot"
            );
        }
    }
}
#[test]
fn conditional_late_overflow_restores_earlier_tap_growth_life_and_all_rng_state() {
    for name in [
        "Aunt May",
        "Dreamdew Entrancer",
        "Fear of Immobility",
        "Thwip!",
        "Wakandan Royal Guard",
    ] {
        let mut g = setup(Player::P0);
        let target = named(
            &mut g,
            if name == "Fear of Immobility" {
                Player::P1
            } else {
                Player::P0
            },
            "Three Tree Mascot",
            Zone::Battlefield,
        );
        let source = named(
            &mut g,
            Player::P0,
            name,
            if name == "Thwip!" {
                Zone::Hand
            } else {
                Zone::Battlefield
            },
        );
        match name {
            "Aunt May" => {
                g.state.objects[target as usize].counters.plus_one = u64::MAX;
                g.move_object(target, Zone::Graveyard);
                g.move_object_raw(target, Zone::Battlefield);
                g.state.objects[target as usize].counters.plus_one = u64::MAX;
                trigger_stack(&mut g, source, None);
                let reference = g.object_ref(target);
                g.state.stack.last_mut().unwrap().entering_object = Some(reference);
            }
            "Thwip!" => {
                cast_stack(&mut g, source, Some(target));
                g.state.players[0].life = i128::MAX;
            }
            _ => {
                trigger_stack(&mut g, source, Some(target));
                if name == "Dreamdew Entrancer" {
                    let top = *g.state.players[0].library.last().unwrap();
                    g.state.objects[top as usize].generation = u64::MAX;
                } else {
                    g.set_counters(
                        target,
                        if name == "Fear of Immobility" {
                            Counters {
                                stun: u64::MAX,
                                ..Counters::default()
                            }
                        } else {
                            Counters {
                                plus_one: u64::MAX - 1,
                                ..Counters::default()
                            }
                        },
                    );
                }
            }
        }
        replay(&mut g, Action::Pass);
        let before = g.snapshot();
        let rev = g.revision();
        assert!(
            matches!(
                g.apply(g.actor().unwrap(), Action::Pass),
                Err(Error::CapacityExceeded(_))
            ),
            "{name}"
        );
        assert_eq!(g.snapshot(), before, "{name}");
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn ward_and_wrong_actors_are_enforced_before_conditional_resolution() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Hoverstone Pilgrim", Zone::Battlefield);
            let source = named(&mut g, p, "Thwip!", Zone::Hand);
            cast_stack(&mut g, source, Some(target));
            for _ in 0..30 {
                if g.observe(g.actor().unwrap()).resolution_payment.is_some() {
                    break;
                }
                choose(&mut g, None);
            }
            assert!(g.observe(g.actor().unwrap()).resolution_payment.is_some());
            let before = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply(g.actor().unwrap().other(), Action::DeclineResolutionMana),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            if pay {
                choose(&mut g, None);
            } else {
                replay(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g, None);
            assert_eq!(g.object_view(target).power, if pay { 4 } else { 2 });
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn queued_entry_effects_survive_source_departure_and_suppressed_entries_do_not_queue() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Dreamdew Entrancer",
            "Fear of Immobility",
            "Little Bear",
            "Wakandan Royal Guard",
        ] {
            let mut g = setup(p);
            let target = named(
                &mut g,
                if name == "Fear of Immobility" {
                    p.other()
                } else {
                    p
                },
                "Three Tree Mascot",
                Zone::Battlefield,
            );
            let source = named(&mut g, p, name, Zone::Battlefield);
            trigger_stack(&mut g, source, Some(target));
            g.move_object(source, Zone::Graveyard);
            settle(&mut g, Some(target));
            match name {
                "Dreamdew Entrancer" => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2)
                }
                "Fear of Immobility" => assert_eq!(g.object_view(target).counters.stun, 1),
                "Little Bear" => assert_eq!(g.object_view(target).counters.plus_one, 1),
                _ => assert_eq!(g.object_view(target).counters.plus_one, 2),
            }
            let mut suppressed = setup(p);
            named(&mut suppressed, p, "Torpor Orb", Zone::Battlefield);
            let source = named(&mut suppressed, p, name, Zone::Hand);
            cast_stack(&mut suppressed, source, None);
            settle(&mut suppressed, None);
            assert_eq!(suppressed.object_view(source).zone, Zone::Battlefield);
            assert_eq!(suppressed.state.players[p.index()].cards_drawn_this_turn, 0);
            assert!(suppressed.state.pending_triggers.is_empty());
        }
    }
}
#[test]
fn little_bear_untap_obeys_stun_replacement_before_conditional_counter() {
    let mut g = setup(Player::P0);
    let target = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[target as usize].tapped = true;
    g.state.objects[target as usize].counters.stun = 2;
    let source = named(&mut g, Player::P0, "Little Bear", Zone::Battlefield);
    trigger_stack(&mut g, source, Some(target));
    settle(&mut g, Some(target));
    let v = g.object_view(target);
    assert!(v.tapped);
    assert_eq!(v.counters.stun, 1);
    assert_eq!(v.counters.plus_one, 1);
}
#[test]
fn divergent_target_choices_own_their_live_relation_and_preserve_the_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Dreamdew Entrancer", Zone::Battlefield);
        let view = g.object_view(source);
        assert!(g.queue_trigger(&view, 0));
        g.prepare_triggers();
        while !matches!(g.state.pending, Pending::Targets { .. }) {
            choose(&mut g, None);
        }
        let parent = g.snapshot();
        let mut yes = g.clone();
        let mut no = g.clone();
        replay(
            &mut yes,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        replay(
            &mut no,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        settle(&mut yes, None);
        settle(&mut no, None);
        assert_eq!(yes.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(no.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(yes.object_view(own).counters.stun, 3);
        assert_eq!(no.object_view(foe).counters.stun, 3);
        assert_eq!(g.snapshot(), parent);
        assert_ne!(yes.snapshot(), no.snapshot());
    }
}
#[test]
fn conditional_temporary_grants_expire_while_life_and_counters_persist() {
    for name in ["Thwip!", "Yip Yip!", "Wakandan Royal Guard"] {
        let mut g = setup(Player::P0);
        let target = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        let source = named(
            &mut g,
            Player::P0,
            name,
            if name.ends_with('!') {
                Zone::Hand
            } else {
                Zone::Battlefield
            },
        );
        if name.ends_with('!') {
            cast_stack(&mut g, source, Some(target));
        } else {
            trigger_stack(&mut g, source, Some(target));
        }
        settle(&mut g, Some(target));
        let life = g.state.players[0].life;
        let counters = g.object_view(target).counters;
        g.finish_cleanup();
        assert_eq!(g.state.objects[target as usize].growth, Stats::default());
        assert!(!g.keywords(target).contains(Keywords::FLYING));
        assert_eq!(g.object_view(target).counters, counters);
        assert_eq!(g.state.players[0].life, life);
    }
}
