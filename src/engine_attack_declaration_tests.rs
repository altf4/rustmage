use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 14] = [
    "Ancestor Dragon",
    "Armasaur Guide",
    "Battlesong Berserker",
    "Boggart Prankster",
    "Jolene, Plundering Pugilist",
    "Lionheart Glimmer",
    "Malamet Brawler",
    "Marshal of the Lost",
    "Meddling Youths",
    "Most Valuable Slayer",
    "Nori, Teller of Tales",
    "Razorkin Hordecaller",
    "Seasoned Consultant",
    "The Chief Warg",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
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
                    Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::ChooseTarget { .. }
                        | Action::ArrangeLibrary { .. }
                )
            })
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
                && g.state.stack.is_empty()
                && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled")
}
fn announce(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..40 {
        if g.state.pending == Pending::Priority {
            return;
        }
        choose(g, target);
    }
    panic!("unannounced")
}
fn declaration(g: &mut Game, p: Player, ids: &[CardId]) {
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
fn fourteen_complete_casts_and_nori_hybrid_options_for_each_seat() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            for option in 0..if name == "Nori, Teller of Tales" {
                2
            } else {
                1
            } {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                g.state.players[p.index()].mana = [30; 6];
                let source = named(&mut g, p, name, Zone::Hand);
                replay(&mut g, Action::AnnounceCast { card: source });
                if name == "Nori, Teller of Tales" {
                    replay(&mut g, Action::ChooseSpellCost { cost: option });
                }
                while g.state.announcing.is_some() {
                    choose(&mut g, None);
                }
                settle(&mut g, None);
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                let indexed = !matches!(name, "Malamet Brawler" | "Nori, Teller of Tales");
                assert_eq!(g.state.attack_sources.contains(&source), indexed, "{name}");
            }
        }
    }
}
#[test]
fn source_need_not_attack_once_per_declaration_for_both_seats_and_every_body() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            let mut g = scenario();
            let source = named(&mut g, p, name, Zone::Battlefield);
            let target = named(&mut g, p, "Dragonlord's Servant", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[b as usize].growth.power = 2;
            let mut ids = vec![target, b, c];
            if matches!(name, "Malamet Brawler" | "Nori, Teller of Tales") {
                ids[2] = source;
            }
            declaration(&mut g, p, &ids);
            let queued = g.state.stack.len()
                + g.state.ordering_triggers.len()
                + g.state.pending_triggers.len()
                + usize::from(g.state.announcing.is_some());
            assert_eq!(queued, 1, "{name}");
            settle(&mut g, Some(target));
            match name {
                "Ancestor Dragon" => assert_eq!(g.state.players[p.index()].life, 23),
                "Armasaur Guide" => assert_eq!(g.object_view(target).counters.plus_one, 1),
                "Battlesong Berserker" => {
                    assert!(g.object_view(target).keywords.contains(Keywords::MENACE))
                }
                "Boggart Prankster" => assert_eq!(g.object_view(target).power, 2),
                "Jolene, Plundering Pugilist" => assert!(
                    g.state
                        .objects
                        .iter()
                        .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Treasure"))
                ),
                "Lionheart Glimmer" => assert_eq!(g.object_view(target).power, 2),
                "Malamet Brawler" => {
                    assert!(g.object_view(target).keywords.contains(Keywords::TRAMPLE))
                }
                "Marshal of the Lost" => assert_eq!(g.object_view(target).power, 4),
                "Meddling Youths" => assert!(
                    g.state
                        .objects
                        .iter()
                        .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
                ),
                "Most Valuable Slayer" | "Nori, Teller of Tales" => assert!(
                    g.object_view(target)
                        .keywords
                        .contains(Keywords::FIRST_STRIKE)
                ),
                "Razorkin Hordecaller" => assert!(
                    g.state
                        .objects
                        .iter()
                        .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Gremlin"))
                ),
                "Seasoned Consultant" => assert_eq!(g.object_view(source).power, 3),
                "The Chief Warg" => {
                    assert_eq!(g.state.players[p.index()].life, 19);
                    assert_eq!(g.state.players[p.index()].library.len(), 19);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn count_threshold_is_event_only_and_empty_selected_or_illegal_attacks_do_not_fire() {
    for name in ["Armasaur Guide", "Meddling Youths", "Seasoned Consultant"] {
        for n in 0..=3 {
            let mut g = scenario();
            named(&mut g, Player::P0, name, Zone::Battlefield);
            let ids: Vec<_> = (0..n)
                .map(|_| named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield))
                .collect();
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            for &id in &ids {
                replay(&mut g, Action::Attack { card: id });
                assert!(g.state.pending_triggers.is_empty());
            }
            if let Some(&id) = ids.first() {
                reject(&mut g, Action::Attack { card: id });
            }
            replay(&mut g, Action::FinishAttackers);
            announce(&mut g, ids.first().copied());
            assert_eq!(g.state.stack.len(), usize::from(n == 3), "{name} n{n}");
            if n == 3 {
                g.move_object(ids[1], Zone::Exile);
                g.move_object(ids[2], Zone::Exile);
                settle(&mut g, ids.first().copied());
                assert!(g.state.stack.is_empty());
            }
        }
    }
}
#[test]
fn power_filter_and_while_guard_differ_and_do_not_recheck() {
    for name in ["Jolene, Plundering Pugilist", "The Chief Warg"] {
        for high_attacks in [false, true] {
            let mut g = scenario();
            named(&mut g, Player::P0, name, Zone::Battlefield);
            let low = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let high = named(&mut g, Player::P0, "Gigantoad", Zone::Battlefield);
            declaration(&mut g, Player::P0, &[if high_attacks { high } else { low }]);
            announce(&mut g, None);
            assert_eq!(
                g.state.stack.len(),
                usize::from(high_attacks || name == "The Chief Warg")
            );
            g.move_object(high, Zone::Exile);
            settle(&mut g, None);
            if name == "The Chief Warg" {
                assert_eq!(g.state.players[0].life, 19);
            }
        }
    }
}
#[test]
fn live_attack_count_is_evaluated_at_resolution_after_departure_and_source_departure() {
    for name in ["Ancestor Dragon", "Marshal of the Lost"] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        declaration(&mut g, Player::P0, &[a, b]);
        announce(&mut g, Some(a));
        g.move_object(b, Zone::Exile);
        g.move_object(source, Zone::Exile);
        settle(&mut g, Some(a));
        assert_eq!(
            g.state.players[0].life,
            if name == "Ancestor Dragon" { 21 } else { 20 }
        );
        assert_eq!(
            g.object_view(a).power,
            if name == "Marshal of the Lost" { 3 } else { 2 }
        );
    }
}
#[test]
fn goblin_target_requires_owned_attacking_subtype_and_rechecks_on_resolution() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Boggart Prankster", Zone::Battlefield);
    let good = named(
        &mut g,
        Player::P0,
        "Dragonlord's Servant",
        Zone::Battlefield,
    );
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(
        &mut g,
        Player::P1,
        "Dragonlord's Servant",
        Zone::Battlefield,
    );
    declaration(&mut g, Player::P0, &[good, bear]);
    assert!(matches!(g.state.pending, Pending::Targets { .. }));
    for id in [source, bear, enemy] {
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
            target: Target::Card(good),
        },
    );
    if g.state.pending != Pending::Priority {
        replay(&mut g, Action::FinishTargets);
    }
    g.move_object(good, Zone::Exile);
    g.move_object(good, Zone::Battlefield);
    settle(&mut g, None);
    assert_eq!(g.object_view(good).power, 1);
}
#[test]
fn attacking_targets_do_not_require_ownership_but_fizzle_when_removed_from_combat() {
    for name in [
        "Malamet Brawler",
        "Most Valuable Slayer",
        "Nori, Teller of Tales",
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        declaration(&mut g, Player::P0, &[source, mate]);
        announce(&mut g, Some(mate));
        g.state.attackers.retain(|a| a.card != mate);
        settle(&mut g, Some(mate));
        assert_eq!(g.object_view(mate).keywords, Keywords::NONE);
        assert_eq!(g.object_view(mate).power, 2);
    }
}
#[test]
fn source_index_and_reentry_are_branch_local_and_never_duplicate() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Ancestor Dragon", Zone::Battlefield);
    let old = g.object_ref(source);
    let saved = g.snapshot();
    let mut fork = g.clone();
    fork.move_object(source, Zone::Exile);
    assert!(!fork.state.attack_sources.contains(&source));
    assert!(g.state.attack_sources.contains(&source));
    fork.move_object(source, Zone::Battlefield);
    assert_ne!(old, fork.object_ref(source));
    assert_eq!(
        fork.state
            .attack_sources
            .iter()
            .filter(|&&id| id == source)
            .count(),
        1
    );
    g.restore(&saved).unwrap();
    assert_eq!(g.object_ref(source), old);
}
#[test]
fn planeswalker_attack_counts_and_opposing_observer_never_triggers() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Ancestor Dragon", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Ancestor Dragon", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let pw = named(
        &mut g,
        Player::P1,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    g.state.objects[pw as usize].counters.loyalty = 4;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    let defender = g.object_ref(pw);
    replay(&mut g, Action::AttackPlaneswalker { card: a, defender });
    replay(&mut g, Action::FinishAttackers);
    announce(&mut g, None);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].object, g.object_ref(own));
    assert_ne!(g.state.stack[0].object, g.object_ref(enemy));
    settle(&mut g, None);
    assert_eq!(g.state.players[0].life, 21);
}
#[test]
fn scaled_life_capacity_failure_rolls_back_entire_resolution() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Ancestor Dragon", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    declaration(&mut g, Player::P0, &[a]);
    announce(&mut g, None);
    replay(&mut g, Action::Pass);
    g.state.players[0].life = i128::MAX;
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}

#[test]
fn jolene_sacrifices_only_a_treasure_as_cost_then_deals_one_damage() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Jolene, Plundering Pugilist",
        Zone::Battlefield,
    );
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    g.create_tokens(Player::P0, factory, 1, false);
    let treasure = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.card == factory)
        .unwrap() as CardId;
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let bear_ref = g.object_ref(bear);
    let treasure_ref = g.object_ref(treasure);
    g.state.players[0].mana = [5; 6];
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    if matches!(g.state.pending, Pending::Targets { .. }) {
        replay(&mut g, Action::FinishTargets);
    }
    reject(&mut g, Action::ChooseAdditionalCost { object: bear_ref });
    replay(
        &mut g,
        Action::ChooseAdditionalCost {
            object: treasure_ref,
        },
    );
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, payment);
    assert_ne!(g.object_view(treasure).zone, Zone::Battlefield);
    settle(&mut g, None);
    assert_eq!(g.state.players[1].life, 19);
}
