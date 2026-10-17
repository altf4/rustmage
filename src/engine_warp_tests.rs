use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 12] = [
    "Bygone Colossus",
    "Eusocial Engineering",
    "Germinating Wurm",
    "Knight Luminary",
    "Red Tiger Mechan",
    "Sinister Cryologist",
    "Starbreach Whale",
    "Weftblade Enhancer",
    "Drix Fatemaker",
    "Mechanozoa",
    "Memorial Team Leader",
    "Nova Hellkite",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [40; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let saved = g.snapshot();
    let rev = g.revision();
    let parent = g.clone();
    let mut child = parent.clone();
    drop(parent);
    act(&mut child, a.clone());
    let expected = child.snapshot();
    act(g, a.clone());
    assert_eq!(g.snapshot(), expected);
    for v in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(v)).unwrap(),
            serde_json::to_value(child.observe(v)).unwrap()
        );
    }
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn pick(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .min_by_key(|a| match a {
            Action::ChooseTarget { .. }
            | Action::ArrangeLibrary {
                destination: LibraryDestination::Graveyard,
                ..
            } => 0,
            Action::PayCosts { .. } | Action::PutTrigger { .. } => 1,
            Action::FinishTargets | Action::FinishAttackers | Action::FinishBlockers => 2,
            Action::Pass => 3,
            Action::Concede => 10000,
            _ => 100,
        })
        .unwrap();
    replay(g, a);
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        pick(g);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn to_end(g: &mut Game) {
    for _ in 0..100 {
        if g.state.step == Step::End {
            return;
        }
        pick(g);
    }
    panic!("no end");
}
fn next_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    for _ in 0..160 {
        if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 && idle(g) {
            return;
        }
        pick(g);
    }
    panic!("no later main");
}
fn paid(g: &mut Game, id: CardId, warp: bool) {
    let p = g.actor().unwrap();
    let delayed_before = g.observe(p).delayed_warp_exiles.len();
    let origin = g.state.objects[id as usize].zone;
    let card = g.state.objects[id as usize].card;
    let before = g.state.players[p.index()]
        .mana
        .iter()
        .map(|n| u64::from(*n))
        .sum::<u64>();
    replay(
        g,
        if warp {
            Action::AnnounceWarp { card: id }
        } else if origin == Zone::Exile {
            Action::AnnounceExile { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    while g.state.announcing.is_some() {
        pick(g);
    }
    let item = g
        .state
        .stack
        .iter()
        .find(|s| s.card == id && s.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(
        item.cast.unwrap().method,
        if warp {
            CastMethod::Warp
        } else {
            CastMethod::Normal
        }
    );
    assert_eq!(item.cast.unwrap().origin, origin);
    assert_eq!(item.mana_value(), card.fixed_mana_value());
    let cost = if warp {
        card.warp_cost().unwrap()
    } else {
        let (c, generic) = card.cost();
        ManaCost {
            colored: c,
            generic,
        }
    };
    assert_eq!(
        before
            - g.state.players[p.index()]
                .mana
                .iter()
                .map(|n| u64::from(*n))
                .sum::<u64>(),
        u64::from(cost.generic) + cost.colored.iter().map(|n| u64::from(*n)).sum::<u64>()
    );
    assert_eq!(g.observe(p).delayed_warp_exiles.len(), delayed_before);
}
#[test]
fn all_twelve_complete_bodies_normal_warp_and_later_paid_recast_both_seats() {
    for p in [Player::P0, Player::P1] {
        for warped in [false, true] {
            for (i, name) in NAMES.iter().enumerate() {
                let mut g = setup(p);
                let ally = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let id = named(&mut g, p, name, Zone::Hand);
                let repeats = if warped { 2 } else { 1 };
                for round in 0..repeats {
                    g.state.players[p.index()].mana = [40; 6];
                    let life = g.state.players[p.index()].life;
                    let lib = g.state.players[p.index()].library.len();
                    let tokens = g
                        .state
                        .objects
                        .iter()
                        .filter(|o| {
                            o.zone == Zone::Battlefield && o.owner == p && o.card.is_token()
                        })
                        .count();
                    let counters = [
                        g.state.objects[ally as usize].counters.plus_one,
                        g.state.objects[enemy as usize].counters.plus_one,
                    ];
                    paid(&mut g, id, warped && round == 0);
                    settle(&mut g);
                    assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
                    match i {
                        0 => assert_eq!((g.stats(id).power, g.stats(id).toughness), (9, 9)),
                        1 => {
                            let land = named(&mut g, p, "Forest", Zone::Hand);
                            replay(&mut g, Action::PlayLand { card: land });
                            settle(&mut g);
                            let robots: Vec<_> = g
                                .state
                                .objects
                                .iter()
                                .enumerate()
                                .filter(|(_, o)| {
                                    o.zone == Zone::Battlefield
                                        && o.owner == p
                                        && o.card.is_token()
                                        && o.card.has_subtype("Robot")
                                })
                                .collect();
                            assert_eq!(robots.len(), tokens + 1);
                            for (j, o) in robots {
                                assert!(o.card.is_artifact());
                                assert_eq!(
                                    (
                                        g.stats(j as CardId).power,
                                        g.stats(j as CardId).toughness,
                                        o.card.colors()
                                    ),
                                    (2, 2, 0)
                                );
                            }
                        }
                        2 => assert_eq!(g.state.players[p.index()].life, life + 2),
                        3 => {
                            assert_eq!(
                                g.state
                                    .objects
                                    .iter()
                                    .filter(|o| o.zone == Zone::Battlefield
                                        && o.owner == p
                                        && o.card.is_token()
                                        && o.card.has_subtype("Human")
                                        && o.card.has_subtype("Soldier"))
                                    .count(),
                                tokens + 1
                            );
                        }
                        4 => assert!(g.keywords(id).contains(Keywords::HASTE)),
                        5 => assert_eq!(g.stats(enemy).power, -1),
                        6 => {
                            assert!(g.keywords(id).contains(Keywords::FLYING));
                            assert_eq!(g.state.players[p.index()].library.len(), lib - 2);
                        }
                        7 => {
                            assert_eq!(
                                g.state.objects[ally as usize].counters.plus_one,
                                counters[0] + 1
                            );
                            assert_eq!(
                                g.state.objects[enemy as usize].counters.plus_one,
                                counters[1] + 1
                            );
                        }
                        8 => {
                            assert_eq!(
                                g.state.objects[ally as usize].counters.plus_one,
                                counters[0] + 1
                            );
                            assert!(g.keywords(ally).contains(Keywords::TRAMPLE));
                            assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
                            assert!(!g.keywords(id).contains(Keywords::TRAMPLE));
                        }
                        9 => {
                            assert!(g.state.objects[enemy as usize].tapped);
                            assert_eq!(g.state.objects[enemy as usize].counters.stun, 1);
                        }
                        10 => {
                            assert_eq!(g.stats(ally).power, 11);
                            assert_eq!(g.stats(id).power, 4);
                            assert_eq!(g.stats(enemy).power, 2);
                        }
                        11 => {
                            assert!(g.keywords(id).contains(Keywords::FLYING));
                            assert!(g.keywords(id).contains(Keywords::HASTE));
                            assert_eq!(g.state.objects[enemy as usize].damage, 1);
                        }
                        _ => unreachable!(),
                    }
                    if warped && round == 0 {
                        let old = g.object_ref(id);
                        let delayed = g.observe(p).delayed_warp_exiles;
                        assert_eq!(delayed.len(), 1);
                        assert_eq!(delayed[0].source, old);
                        assert_eq!(delayed, g.observe(p.other()).delayed_warp_exiles);
                        to_end(&mut g);
                        settle(&mut g);
                        assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
                        let permission = g.observe(p).warped_cards;
                        assert_eq!(permission.len(), 1);
                        assert_eq!(permission, g.observe(p.other()).warped_cards);
                        assert_eq!(permission[0].object, g.object_ref(id));
                        assert_eq!(permission[0].exiled_turn, g.state.turn);
                        reject(&mut g, Action::AnnounceExile { card: id });
                        reject(&mut g, Action::AnnounceWarp { card: id });
                        next_main(&mut g, p);
                    } else {
                        assert!(g.observe(p).delayed_warp_exiles.is_empty());
                        assert!(g.observe(p).warped_cards.is_empty());
                    }
                }
            }
        }
    }
}
#[test]
fn countered_warp_spell_has_no_delayed_exile_or_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        paid(&mut g, id, true);
        let stack = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(stack, false));
        settle(&mut g);
        to_end(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert!(g.state.warp.is_none());
    }
}
#[test]
fn delayed_ability_is_counterable_once_and_independent_of_source() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
            paid(&mut g, id, true);
            settle(&mut g);
            let old = g.object_ref(id);
            if !counter {
                g.move_object(id, Zone::Hand);
                g.move_object(id, Zone::Battlefield);
                assert_ne!(old, g.object_ref(id));
            }
            to_end(&mut g);
            while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                pick(&mut g);
            }
            let item = g
                .state
                .stack
                .iter()
                .find(|s| s.kind == StackKind::DelayedWarpExile)
                .unwrap()
                .clone();
            assert_eq!(item.delayed_warp_exile.as_ref().unwrap().source, old);
            assert_ne!(item.id, item.delayed_warp_exile.as_ref().unwrap().id);
            if counter {
                assert!(g.counter_stack_item(item.id, false));
            }
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert!(g.state.warp.is_none());
            g.begin_step(Step::End);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        }
    }
}
#[test]
fn warped_permission_is_not_free_not_warp_not_plot_and_never_inherited() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Bygone Colossus", Zone::Hand);
        paid(&mut g, id, true);
        settle(&mut g);
        to_end(&mut g);
        settle(&mut g);
        next_main(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceExile { card: id });
        reject(&mut g, Action::AnnounceWarp { card: id });
        reject(&mut g, Action::AnnouncePlotted { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 9];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: id })
        );
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Exile);
        assert_ne!(g.object_ref(id), old);
        assert!(g.observe(p).warped_cards.is_empty());
        reject(&mut g, Action::AnnounceExile { card: id });
    }
}

#[test]
fn real_joint_mana_sacrifice_funds_warp_without_double_using_the_same_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let before = g.snapshot();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceWarp { card: id })
        );
        assert_eq!(g.snapshot(), before);
        reject(&mut g, Action::AnnounceWarp { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let source = g.object_ref(id);
        replay(&mut g, Action::AnnounceWarp { card: id });
        assert_ne!(g.object_ref(id), source);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==pizza&&production[4]==1)).unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        settle(&mut g);
        assert_eq!(g.observe(p).delayed_warp_exiles.len(), 1);
    }
}
#[test]
fn normal_timing_wrong_actor_and_real_warp_funding_queries_are_pure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        for step in [Step::Upkeep, Step::BeginCombat, Step::End] {
            g.state.step = step;
            reject(&mut g, Action::AnnounceWarp { card: id });
        }
        g.state.step = Step::Main1;
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceWarp { card: id });
        g.state.active = p;
        let before = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(p.other(), Action::AnnounceWarp { card: id }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        g.state.players[p.index()].mana = [20, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceWarp { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
        let before = g.snapshot();
        let a = g.legal_actions();
        assert_eq!(g.legal_actions(), a);
        assert!(a.contains(&Action::AnnounceWarp { card: id }));
        assert!(!a.contains(&Action::AnnounceCast { card: id }));
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn two_warp_casts_of_same_card_keep_distinct_delayed_subjects_and_one_real_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        paid(&mut g, id, true);
        settle(&mut g);
        let first = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        paid(&mut g, id, true);
        settle(&mut g);
        let second = g.object_ref(id);
        assert_ne!(first, second);
        let regs = g.observe(p).delayed_warp_exiles;
        assert_eq!(regs.len(), 2);
        assert_ne!(regs[0].id, regs[1].id);
        assert_eq!(regs[0].source, first);
        assert_eq!(regs[1].source, second);
        to_end(&mut g);
        settle(&mut g);
        let cards = g.observe(p).warped_cards;
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].object, g.object_ref(id));
        assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
    }
}
#[test]
fn queued_warp_delayed_ability_does_not_follow_a_later_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        paid(&mut g, id, true);
        settle(&mut g);
        let old = g.object_ref(id);
        to_end(&mut g);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            pick(&mut g);
        }
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.kind == StackKind::DelayedWarpExile)
        );
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        assert_ne!(g.object_ref(id), old);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert!(g.state.warp.is_none());
    }
}
#[test]
fn warp_created_after_end_started_waits_for_next_end_and_gates_actual_exile_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.begin_step(Step::End);
        let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
        g.state.objects[id as usize].gained_keywords = Keywords::FLASH;
        let cast_turn = g.state.turn;
        paid(&mut g, id, true);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.observe(p).delayed_warp_exiles.len(), 1);
        for _ in 0..150 {
            if g.state.turn > cast_turn && g.state.step == Step::End {
                break;
            }
            pick(&mut g);
        }
        assert!(g.state.turn > cast_turn);
        settle(&mut g);
        let card = g.observe(p).warped_cards[0];
        assert!(card.exiled_turn > cast_turn);
        assert_eq!(card.exiled_turn, g.state.turn);
        assert!(!g.can_cast_warped(p, id));
        next_main(&mut g, p);
        assert!(g.can_cast_warped(p, id));
    }
}
#[test]
fn registration_queue_and_late_exile_capacity_failures_restore_owned_state() {
    for p in [Player::P0, Player::P1] {
        for stage in 0..3 {
            let mut g = setup(p);
            let id = named(&mut g, p, "Germinating Wurm", Zone::Hand);
            paid(&mut g, id, true);
            if stage == 0 {
                replay(&mut g, Action::Pass);
                g.state.next_stack_id = u64::MAX;
            } else {
                settle(&mut g);
                if stage == 1 {
                    g.state.step = Step::Main2;
                    replay(&mut g, Action::Pass);
                    g.state.next_stack_id = u64::MAX;
                } else {
                    to_end(&mut g);
                    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                        pick(&mut g);
                    }
                    g.state.objects[id as usize].generation = u64::MAX;
                    let item = g
                        .state
                        .stack
                        .iter_mut()
                        .find(|s| s.kind == StackKind::DelayedWarpExile)
                        .unwrap();
                    item.delayed_warp_exile.as_mut().unwrap().source.generation = u64::MAX;
                    replay(&mut g, Action::Pass);
                }
            }
            let before = g.snapshot();
            let rev = g.revision();
            let actor = g.actor().unwrap();
            assert!(
                matches!(
                    g.apply(actor, Action::Pass),
                    Err(Error::CapacityExceeded(_))
                ),
                "stage{stage}"
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), rev);
        }
    }
}
#[test]
fn conditional_static_grants_follow_source_counters_and_live_turn_then_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let drix = named(&mut g, p, "Drix Fatemaker", Zone::Hand);
        paid(&mut g, drix, true);
        settle(&mut g);
        assert!(g.keywords(ally).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.state.objects[ally as usize].counters.plus_one = 0;
        assert!(!g.keywords(ally).contains(Keywords::TRAMPLE));
        g.state.objects[ally as usize].counters.plus_one = 1;
        g.move_object(drix, Zone::Exile);
        assert!(!g.keywords(ally).contains(Keywords::TRAMPLE));
        assert!(g.observe(p).warped_cards.is_empty());
        let leader = named(&mut g, p, "Memorial Team Leader", Zone::Hand);
        paid(&mut g, leader, false);
        settle(&mut g);
        assert_eq!(g.stats(ally).power, 4);
        g.state.active = p.other();
        assert_eq!(g.stats(ally).power, 3);
        assert_eq!(g.stats(enemy).power, 2);
        g.state.active = p;
        assert_eq!(g.stats(ally).power, 4);
        g.move_object(leader, Zone::Graveyard);
        assert_eq!(g.stats(ally).power, 3);
    }
}
#[test]
fn both_collateral_complete_bodies_keep_flash_cycling_and_intervening_if() {
    for p in [Player::P0, Player::P1] {
        for artifact in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
            if artifact {
                named(&mut g, p, "Candy Trail", Zone::Battlefield);
            }
            let id = named(&mut g, p, "Waylaying Pirates", Zone::Hand);
            paid(&mut g, id, false);
            settle(&mut g);
            assert_eq!(g.state.objects[target as usize].tapped, artifact);
            assert_eq!(
                g.state.objects[target as usize].counters.stun,
                u64::from(artifact)
            );
        }
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let id = named(&mut g, p, "Runaway Boulder", Zone::Hand);
        g.state.step = Step::End;
        paid(&mut g, id, false);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].damage, 6);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        let cycle = named(&mut g, p, "Runaway Boulder", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: cycle,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.state.announcing.is_some() {
            pick(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[cycle as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
    }
}

#[test]
fn sparse_warp_storage_is_owned_and_reclaims_after_paid_recast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Bygone Colossus", Zone::Hand);
        assert!(g.state.warp.is_none());
        let baseline = g.state_bytes();
        paid(&mut g, id, true);
        settle(&mut g);
        let state = g.state.warp.as_ref().unwrap();
        assert_eq!(
            state.bytes(),
            std::mem::size_of::<warp::WarpState>() + std::mem::size_of::<warp::DelayedWarp>()
        );
        let registered = g.state_bytes();
        let snap = g.snapshot();
        let parent = g.clone();
        let mut child = parent.clone();
        drop(parent);
        g.move_object(id, Zone::Hand);
        to_end(&mut child);
        settle(&mut child);
        assert_eq!(child.state.objects[id as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
        g.restore(&snap).unwrap();
        assert_eq!(g.state_bytes(), registered);
        to_end(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.warp.as_ref().unwrap().bytes(),
            std::mem::size_of::<warp::WarpState>() + std::mem::size_of::<WarpedCard>()
        );
        next_main(&mut g, p);
        g.state.players[p.index()].mana = [40; 6];
        paid(&mut g, id, false);
        settle(&mut g);
        assert!(g.state.warp.is_none());
        println!(
            "warp {p:?} baseline={baseline} registered={registered} box={} delayed={} permission={}",
            std::mem::size_of::<warp::WarpState>(),
            std::mem::size_of::<warp::DelayedWarp>(),
            std::mem::size_of::<WarpedCard>()
        );
    }
}
