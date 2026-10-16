use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 6] = [
    "EPF Point Squad",
    "East Wind Avatar",
    "Mighty Mutanimals",
    "Mutant Town Musicians",
    "Raphael, Tough Turtle",
    "Slash, Reptile Rampager",
];
const BASE: [(i128, i128); 6] = [(2, 1), (2, 4), (2, 1), (2, 4), (1, 3), (7, 5)];
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
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
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
                    Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                        | Action::Pass
                )
            })
        })
        .expect("fixture choice")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn full_cast(g: &mut Game, id: CardId, target: Option<CardId>) {
    replay(g, Action::AnnounceCast { card: id });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(g, target);
    }
    settle(g, target);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
}
fn tokens(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Mutant")
        })
        .count()
}
fn mutant() -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Mutant"))
        .unwrap()
}

#[test]
fn all_six_bodies_actually_cast_for_both_seats_and_trigger_only_for_another_owned_creature() {
    for p in [Player::P0, Player::P1] {
        for i in 0..6 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Hand);
            full_cast(&mut g, id, Some(id));
            let initial = u64::from(i == 2);
            assert_eq!(g.object_view(id).counters.plus_one, initial);
            assert_eq!(
                pt(&g, id),
                (
                    BASE[i].0 + i128::from(initial),
                    BASE[i].1 + i128::from(initial)
                )
            );
            assert_eq!(tokens(&g, p), usize::from(i == 2));
            assert_eq!(g.state.players[p.other().index()].life, 20);
            let intrinsic = [
                Keywords::NONE,
                Keywords::FLYING.union(Keywords::VIGILANCE),
                Keywords::NONE,
                Keywords::TRAMPLE,
                Keywords::NONE,
                Keywords::NONE,
            ][i];
            assert_eq!(g.keywords(id), intrinsic);
            for (owner, name) in [(p, "Plains"), (p, "Goldvein Pick"), (p.other(), "Bear Cub")] {
                let x = named(&mut g, owner, name, Zone::Hand);
                g.move_object(x, Zone::Battlefield);
                settle(&mut g, Some(id));
            }
            assert_eq!(g.object_view(id).counters.plus_one, initial);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            full_cast(&mut g, bear, Some(id));
            assert_eq!(
                g.object_view(id).counters.plus_one,
                initial + u64::from(i == 0 || i == 2)
            );
            assert_eq!(
                pt(&g, id),
                (
                    BASE[i].0 + i128::from(initial) + i128::from(i <= 3),
                    BASE[i].1 + i128::from(initial) + i128::from(i == 0 || i == 2)
                )
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - if i == 4 {
                    1
                } else if i == 5 {
                    2
                } else {
                    0
                }
            );
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn epf_hybrid_announcements_pay_each_exact_colored_option_and_reject_unfunded_cast() {
    for p in [Player::P0, Player::P1] {
        for option in 0..3 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[0], Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, Action::AnnounceCast { card: id });
            g.state.players[p.index()].mana = [2, 0, 0, 2, 0, 1];
            replay(&mut g, Action::AnnounceCast { card: id });
            let options: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a, Action::ChooseSpellCost { .. }))
                .collect();
            assert_eq!(options.len(), 3);
            replay(&mut g, options[option].clone());
            choose(&mut g, None);
            settle(&mut g, None);
            let mana = g.state.players[p.index()].mana;
            assert_eq!(mana[0] + mana[3], 2);
            assert_eq!(mana[5], 0);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn each_creature_in_a_token_batch_triggers_alliance_without_a_nontoken_restriction() {
    for p in [Player::P0, Player::P1] {
        for i in 0..6 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Hand);
            full_cast(&mut g, id, Some(id));
            let counters = g.object_view(id).counters.plus_one;
            g.create_tokens(p, mutant(), 2, false);
            settle(&mut g, Some(id));
            assert_eq!(
                g.object_view(id).counters.plus_one,
                counters + if i == 0 || i == 2 { 2 } else { 0 }
            );
            assert_eq!(
                pt(&g, id).0,
                BASE[i].0 + i128::from(counters) + if i <= 3 { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - if i == 4 {
                    2
                } else if i == 5 {
                    4
                } else {
                    0
                }
            );
        }
    }
}
#[test]
fn simultaneous_newcomers_see_each_other_and_mighty_token_but_never_themselves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let epf = named(&mut g, p, NAMES[0], Zone::Hand);
        let mighty = named(&mut g, p, NAMES[2], Zone::Hand);
        let events = g.move_objects_raw(&[epf, mighty], Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        settle(&mut g, Some(mighty));
        assert_eq!(g.object_view(epf).counters.plus_one, 2);
        assert_eq!(g.object_view(mighty).counters.plus_one, 2);
        assert_eq!(tokens(&g, p), 1);
    }
}
#[test]
fn source_reentry_does_not_receive_old_source_effects_but_damage_and_target_effects_survive() {
    for p in [Player::P0, Player::P1] {
        for i in 0..6 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
            g.prepare_triggers();
            // Commit any required target before removing the original source.
            for _ in 0..10 {
                if g.state.pending == Pending::Priority {
                    break;
                }
                choose(&mut g, Some(bear));
            }
            assert_eq!(g.state.stack.len(), 1);
            let old = g.object_ref(id);
            assert_eq!(g.state.stack[0].object, old);
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            assert_ne!(old, g.object_ref(id));
            settle(&mut g, Some(id));
            let expected = u64::from(i == 2);
            assert_eq!(g.object_view(id).counters.plus_one, expected);
            assert_eq!(
                pt(&g, id),
                (
                    BASE[i].0 + i128::from(expected),
                    BASE[i].1 + i128::from(expected)
                )
            );
            assert_eq!(g.object_view(bear).counters.plus_one, u64::from(i == 2));
            assert_eq!(
                g.state.players[p.other().index()].life,
                20 - if i == 4 {
                    1
                } else if i == 5 {
                    2
                } else {
                    0
                }
            );
        }
    }
}
#[test]
fn mighty_requires_an_owned_creature_and_revalidates_exact_target_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[2], Zone::Battlefield);
        let land = named(&mut g, p, "Plains", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        g.prepare_triggers();
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while g.state.pending != Pending::Priority {
            choose(&mut g, None);
        }
        let old = g.object_ref(bear);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        // Reentry produces a separate new trigger; it chooses Mighty.
        settle(&mut g, Some(id));
        assert_ne!(old, g.object_ref(bear));
        assert_eq!(g.object_view(bear).counters.plus_one, 0);
        assert_eq!(g.object_view(id).counters.plus_one, 1);
    }
}
#[test]
fn temporary_alliance_growth_expires_while_counters_and_intrinsics_remain() {
    let mut g = setup(Player::P0);
    let ids: Vec<_> = (0..4)
        .map(|i| named(&mut g, Player::P0, NAMES[i], Zone::Battlefield))
        .collect();
    g.create_tokens(Player::P0, mutant(), 2, false);
    settle(&mut g, Some(ids[2]));
    assert_eq!(pt(&g, ids[0]), (4, 3));
    assert_eq!(pt(&g, ids[1]), (4, 4));
    assert_eq!(pt(&g, ids[2]), (4, 3));
    assert_eq!(pt(&g, ids[3]), (4, 4));
    g.finish_cleanup();
    assert_eq!(pt(&g, ids[0]), (4, 3));
    assert_eq!(pt(&g, ids[1]), (2, 4));
    assert_eq!(pt(&g, ids[2]), (4, 3));
    assert_eq!(pt(&g, ids[3]), (2, 4));
    assert!(g.keywords(ids[1]).contains(Keywords::VIGILANCE));
    assert!(g.keywords(ids[3]).contains(Keywords::TRAMPLE));
}
#[test]
fn counter_overflow_in_a_real_alliance_resolution_is_atomic() {
    for i in [0, 2] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, NAMES[i], Zone::Battlefield);
        g.state.objects[id as usize].counters.plus_one = u64::MAX;
        g.create_tokens(Player::P0, mutant(), 1, false);
        g.prepare_triggers();
        while g.state.pending != Pending::Priority {
            choose(&mut g, Some(id));
        }
        replay(&mut g, Action::Pass);
        let s = g.snapshot();
        let r = g.revision();
        let p = g.actor().unwrap();
        assert!(matches!(
            g.apply(p, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
    }
}
#[test]
fn slash_attacks_create_a_mutant_then_its_alliance_deals_two_without_combat_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[5], Zone::Hand);
        full_cast(&mut g, id, None);
        g.begin_turn();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
        settle(&mut g, None);
        assert_eq!(tokens(&g, p), 1);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert!(g.object_view(id).tapped);
    }
}
#[test]
fn simultaneous_owned_and_opposing_entries_place_alliance_triggers_in_apnap_order() {
    for active in [Player::P0, Player::P1] {
        let mut g = setup(active);
        let mut ids = Vec::new();
        let mut bears = Vec::new();
        for p in [Player::P0, Player::P1] {
            ids.push(named(&mut g, p, NAMES[0], Zone::Battlefield));
            ids.push(named(&mut g, p, NAMES[1], Zone::Battlefield));
            bears.push(named(&mut g, p, "Bear Cub", Zone::Hand));
        }
        let events = g.move_objects_raw(&bears, Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        g.prepare_triggers();
        assert_eq!(g.actor(), Some(active));
        assert_eq!(g.observe(active).waiting_triggers.len(), 4);
        assert_eq!(
            g.legal_actions()
                .iter()
                .filter(|a| matches!(a, Action::PutTrigger { .. }))
                .count(),
            2
        );
        for _ in 0..8 {
            if g.state.pending == Pending::Priority {
                break;
            }
            choose(&mut g, None);
        }
        assert_eq!(g.state.stack.len(), 4);
        assert!(g.state.stack[..2].iter().all(|s| s.controller == active));
        assert!(
            g.state.stack[2..]
                .iter()
                .all(|s| s.controller == active.other())
        );
        settle(&mut g, None);
        for pair in ids.chunks(2) {
            assert_eq!(g.object_view(pair[0]).counters.plus_one, 1);
            assert_eq!(pt(&g, pair[1]), (3, 4));
        }
    }
}
