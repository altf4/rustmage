use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 15] = [
    "Cartographer's Companion",
    "Cenote Scout",
    "Fanatical Offering",
    "Get Lost",
    "Kinjalli's Dawnrunner",
    "Miner's Guidewing",
    "Over the Edge",
    "Pathfinding Axejaw",
    "River Herald Guide",
    "River Herald Scout",
    "Seeker of Sunlight",
    "Sentinel of the Nameless City",
    "Spyglass Siren",
    "Waterwind Scout",
    "Merfolk Cave-Diver",
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
        s.mana = [40; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let end = g.snapshot();
    assert_eq!(f.snapshot(), end);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), end);
}
fn reject(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.revision(), rev);
}
fn choose(g: &mut Game, target: Option<CardId>, mode: u16, graveyard: bool) {
    let aa = g.legal_actions();
    let a = target
        .and_then(|id| {
            aa.iter().find(|a| {
                **a == Action::ChooseTarget {
                    target: Target::Card(id),
                }
            })
        })
        .or_else(|| {
            aa.iter()
                .find(|a| matches!(a,Action::ChooseMode{mode:m}if *m==mode))
        })
        .or_else(|| {
            aa.iter()
                .find(|a| matches!(a,Action::ChooseExplore{graveyard:b,..}if *b==graveyard))
        })
        .or_else(|| {
            aa.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseAdditionalCost { .. }
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::Pass
                )
            })
        })
        .unwrap_or_else(|| panic!("{:?}: {aa:?}", g.state.pending));
    replay(g, a.clone());
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.announcing.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game, target: Option<CardId>, graveyard: bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        choose(g, target, 0, graveyard);
    }
    panic!("unsettled");
}
fn cast_stack(g: &mut Game, source: CardId, target: Option<CardId>, mode: u16) {
    replay(g, Action::AnnounceCast { card: source });
    while g.state.announcing.is_some() {
        choose(g, target, mode, false);
    }
}
fn enter_choice(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Explore { .. }) {
            return;
        }
        assert!(!idle(g), "no explore choice");
        choose(g, None, 0, false);
    }
    panic!("no choice");
}
fn activate(g: &mut Game, source: CardId, target: Option<CardId>) {
    replay(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        choose(g, target, 0, false);
    }
}
fn map_card() -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Map"))
        .unwrap()
}
fn maps(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.owner == p && o.card == map_card())
                .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn all_fifteen_full_bodies_cast_with_printed_characteristics_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            let mut g = setup(p);
            let victim = named(
                &mut g,
                p.other(),
                if n == "Over the Edge" {
                    "Bear Trap"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, n, Zone::Hand);
            cast_stack(&mut g, source, Some(victim), 0);
            settle(&mut g, Some(victim), false);
            assert_eq!(
                g.object_view(source).zone,
                if matches!(n, "Fanatical Offering" | "Get Lost" | "Over the Edge") {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
                "{n}"
            );
            if n == "Get Lost" {
                assert_eq!(maps(&g, p.other()).len(), 2);
            }
            if matches!(
                n,
                "Cartographer's Companion"
                    | "Sentinel of the Nameless City"
                    | "Spyglass Siren"
                    | "Waterwind Scout"
                    | "Fanatical Offering"
            ) {
                assert_eq!(maps(&g, p).len(), 1);
            }
            if matches!(
                n,
                "Cenote Scout"
                    | "Kinjalli's Dawnrunner"
                    | "Pathfinding Axejaw"
                    | "River Herald Guide"
                    | "River Herald Scout"
            ) {
                assert_eq!(
                    g.observe(p).last_explore.unwrap().destination,
                    Some(Zone::Hand)
                );
            }
            assert_eq!(g.observe(p).schema, 63);
        }
    }
}
#[test]
fn nonland_reveal_is_public_counter_precedes_choice_and_event_follows_exact_commit() {
    for p in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let mut g = setup(p);
            let top = named(&mut g, p, "Bear Cub", Zone::Library);
            let source = named(&mut g, p, "Cenote Scout", Zone::Hand);
            cast_stack(&mut g, source, None, 0);
            let old = g.object_ref(top);
            let rng = g.state.rng;
            enter_choice(&mut g);
            let reference = g.object_ref(source);
            assert_eq!(g.object_view(source).counters.plus_one, 1);
            assert_eq!(g.object_ref(top), old);
            assert_eq!(g.state.rng, rng);
            for viewer in [p, p.other()] {
                let o = g.observe(viewer);
                let c = o.explore_choice.unwrap();
                assert_eq!(c.explorer.object, reference);
                assert_eq!(c.revealed.object, old);
                assert_eq!(c.revealed.card.name(), "Bear Cub");
                assert!(c.counter_added);
                assert!(o.last_explore.is_none());
                assert_eq!(o.known_library_top[p.index()].unwrap().object, old);
                assert!(o.library_choice.is_none());
            }
            assert!(g.observe(p.other()).actions.is_empty());
            let before = g.snapshot();
            assert_eq!(
                g.apply(
                    p.other(),
                    Action::ChooseExplore {
                        object: old,
                        graveyard
                    }
                ),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before);
            reject(
                &mut g,
                Action::ChooseExplore {
                    object: ObjectRef {
                        card: old.card,
                        generation: old.generation + 1,
                    },
                    graveyard,
                },
            );
            reject(&mut g, Action::Pass);
            replay(
                &mut g,
                Action::ChooseExplore {
                    object: old,
                    graveyard,
                },
            );
            let event = g.observe(p).last_explore.unwrap();
            assert_eq!(event.sequence, 1);
            assert_eq!(event.explorer.object, reference);
            assert!(event.counter_added);
            assert_eq!(
                event.destination,
                Some(if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Library
                })
            );
            assert_eq!(
                event.result.unwrap().generation,
                old.generation + u64::from(graveyard)
            );
            assert_eq!(g.state.rng, rng);
            assert_eq!(
                g.observe(p).known_library_top[p.index()].is_none(),
                graveyard
            );
        }
    }
}
#[test]
fn land_put_into_hand_is_public_new_incarnation_not_a_draw_and_empty_still_explores() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            if empty {
                let ids = g.state.players[p.index()].library.clone();
                for id in ids {
                    g.move_object(id, Zone::Graveyard);
                }
            }
            let source = named(&mut g, p, "Cenote Scout", Zone::Hand);
            let top = g.state.players[p.index()]
                .library
                .last()
                .map(|&id| g.object_ref(id));
            let draw = g.state.players[p.index()].cards_drawn_this_turn;
            cast_stack(&mut g, source, None, 0);
            settle(&mut g, None, false);
            let e = g.observe(p).last_explore.unwrap();
            assert_eq!(e.counter_added, empty);
            assert_eq!(e.destination, if empty { None } else { Some(Zone::Hand) });
            assert_eq!(e.revealed.map(|r| r.object), top);
            assert_eq!(g.object_view(source).counters.plus_one, u64::from(empty));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draw);
            assert!(!g.state.players[p.index()].attempted_empty_draw);
            if let Some(old) = top {
                let new = g.object_ref(old.card);
                assert_eq!(new.generation, old.generation + 1);
                for viewer in [p, p.other()] {
                    assert!(g.observe(viewer).known_hand.contains(&new));
                }
            }
        }
    }
}
#[test]
fn source_departure_and_reentry_still_explores_old_object_without_countering_new_one() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Bear Cub", Zone::Library);
            let source = named(&mut g, p, "Cenote Scout", Zone::Hand);
            cast_stack(&mut g, source, None, 0);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            let old = g.object_ref(source);
            assert!(g.state.stack.iter().any(|s| s.object == old));
            g.move_object(source, Zone::Graveyard);
            if reenter {
                // Isolate the old pending ability; raw reentry intentionally omits a second ETB trigger.
                g.move_object_raw(source, Zone::Battlefield);
            }
            enter_choice(&mut g);
            let choice = g.observe(p).explore_choice.unwrap();
            assert_eq!(choice.explorer.object, old);
            assert_eq!(choice.explorer.controller, p);
            assert!(!choice.counter_added);
            let top = choice.revealed.object;
            replay(
                &mut g,
                Action::ChooseExplore {
                    object: top,
                    graveyard: false,
                },
            );
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            assert_eq!(g.observe(p).last_explore.unwrap().explorer.object, old);
        }
    }
}

#[test]
fn map_full_cost_target_timing_source_sacrifice_and_normal_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[own as usize].tapped = true;
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let map = spawn(&mut g, p, map_card(), Zone::Battlefield);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: map,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(map),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        while g.state.announcing.is_some() {
            choose(&mut g, None, 0, false);
        }
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - 1
        );
        assert_eq!(g.object_view(map).zone, Zone::Outside);
        assert!(g.observe(p).last_explore.is_none());
        settle(&mut g, None, false);
        assert_eq!(
            g.observe(p).last_explore.unwrap().explorer.object,
            g.object_ref(own)
        );
        assert!(g.object_view(own).tapped);
        for step in [Step::Upkeep, Step::BeginCombat, Step::End] {
            let mut g = setup(p);
            g.state.step = step;
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let map = spawn(&mut g, p, map_card(), Zone::Battlefield);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==map))
            );
        }
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let map = spawn(&mut g, p, map_card(), Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==map))
        );
        g.state.players[p.index()].mana = [40; 6];
        g.state.objects[map as usize].tapped = true;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==map))
        );
    }
}
#[test]
fn targeted_map_explore_fizzles_on_departure_and_reincarnation_after_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let map = spawn(&mut g, p, map_card(), Zone::Battlefield);
        activate(&mut g, map, Some(own));
        g.move_object(own, Zone::Graveyard);
        g.move_object(own, Zone::Battlefield);
        let library = g.state.players[p.index()].library.clone();
        settle(&mut g, None, false);
        assert!(g.observe(p).last_explore.is_none());
        assert_eq!(g.state.players[p.index()].library, library);
        assert_eq!(g.object_view(own).counters.plus_one, 0);
        assert_eq!(g.object_view(map).zone, Zone::Outside);
    }
}
#[test]
fn double_explore_is_sequential_keep_same_ref_or_bin_then_land_and_triggers_wait() {
    for p in [Player::P0, Player::P1] {
        for first_bin in [false, true] {
            let mut g = setup(p);
            let diver = named(&mut g, p, "Merfolk Cave-Diver", Zone::Battlefield);
            let explorer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let top = named(&mut g, p, "Bear Cub", Zone::Library);
            let original = g.object_ref(top);
            let source = named(&mut g, p, "Over the Edge", Zone::Hand);
            cast_stack(&mut g, source, Some(explorer), 1);
            enter_choice(&mut g);
            assert_eq!(g.object_view(explorer).counters.plus_one, 1);
            assert_eq!(g.object_view(diver).power, 2);
            replay(
                &mut g,
                Action::ChooseExplore {
                    object: original,
                    graveyard: first_bin,
                },
            );
            if !first_bin {
                assert_eq!(
                    g.observe(p).explore_choice.unwrap().revealed.object,
                    original
                );
                assert_eq!(g.object_view(explorer).counters.plus_one, 2);
                assert_eq!(g.observe(p).last_explore.unwrap().sequence, 1);
                assert_eq!(g.object_view(diver).power, 2);
                replay(
                    &mut g,
                    Action::ChooseExplore {
                        object: original,
                        graveyard: false,
                    },
                );
            }
            settle(&mut g, None, false);
            assert_eq!(g.observe(p).last_explore.unwrap().sequence, 2);
            assert_eq!(
                g.object_view(explorer).counters.plus_one,
                if first_bin { 1 } else { 2 }
            );
            assert_eq!(g.object_view(diver).power, 4);
            assert!(
                g.object_view(diver)
                    .keywords
                    .contains(Keywords::CANT_BE_BLOCKED)
            );
            assert_eq!(
                g.object_view(top).zone,
                if first_bin {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            g.finish_cleanup();
            assert_eq!(g.object_view(diver).power, 2);
            assert!(
                !g.object_view(diver)
                    .keywords
                    .contains(Keywords::CANT_BE_BLOCKED)
            );
        }
    }
}
#[test]
fn cave_diver_observes_empty_and_departed_explorer_but_not_countering_or_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let diver = named(&mut g, p, "Merfolk Cave-Diver", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Merfolk Cave-Diver", Zone::Battlefield);
        g.state.objects[diver as usize].counters.plus_one = 1;
        assert!(g.state.pending_triggers.is_empty());
        let ids = g.state.players[p.index()].library.clone();
        for id in ids {
            g.move_object(id, Zone::Graveyard);
        }
        let source = named(&mut g, p, "Cenote Scout", Zone::Hand);
        cast_stack(&mut g, source, None, 0);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        let old = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g, None, false);
        let event = g.observe(p).last_explore.unwrap();
        assert_eq!(event.explorer.object, old);
        assert!(!event.counter_added);
        assert!(event.revealed.is_none());
        assert_eq!(g.object_view(diver).power, 4);
        assert_eq!(g.object_view(enemy).power, 2);
        assert!(
            g.state
                .explore_data
                .as_ref()
                .unwrap()
                .sources
                .contains(&diver)
        );
        g.move_object(diver, Zone::Graveyard);
        assert!(
            !g.state
                .explore_data
                .as_ref()
                .unwrap()
                .sources
                .contains(&diver)
        );
    }
}
#[test]
fn get_lost_uses_old_controller_even_when_indestructible_and_creates_nothing_on_fizzle() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            for indestructible in [false, true] {
                let mut g = setup(p);
                let owner = if own { p } else { p.other() };
                let victim = named(&mut g, owner, "Bear Cub", Zone::Battlefield);
                if indestructible {
                    g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
                }
                let source = named(&mut g, p, "Get Lost", Zone::Hand);
                cast_stack(&mut g, source, Some(victim), 0);
                settle(&mut g, None, false);
                assert_eq!(
                    g.object_view(victim).zone,
                    if indestructible {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(maps(&g, owner).len(), 2);
                assert!(maps(&g, owner.other()).is_empty());
            }
        }
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Get Lost", Zone::Hand);
        cast_stack(&mut g, source, Some(victim), 0);
        g.move_object(victim, Zone::Hand);
        settle(&mut g, None, false);
        assert!(maps(&g, p).is_empty());
        assert!(maps(&g, p.other()).is_empty());
    }
}

#[test]
fn revealed_top_draw_preserves_hand_knowledge_but_private_scry_and_shuffle_forget_position() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Bear Cub", Zone::Library);
        let source = named(&mut g, p, "Cenote Scout", Zone::Hand);
        cast_stack(&mut g, source, None, 0);
        settle(&mut g, None, false);
        let historical = g.observe(p).last_explore.clone();
        let saved = g.snapshot();
        g.draw(p);
        let reference = g.object_ref(top);
        for viewer in [p, p.other()] {
            let o = g.observe(viewer);
            assert!(o.known_hand.contains(&reference));
            assert!(o.known_library_top[p.index()].is_none());
            assert_eq!(o.last_explore, historical);
        }
        g.restore(&saved).unwrap();
        let opt = named(&mut g, p, "Opt", Zone::Hand);
        cast_stack(&mut g, opt, None, 0);
        while !matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            choose(&mut g, None, 0, false);
        }
        assert_eq!(g.public_library_top(p), Some(g.object_ref(top)));
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g, None, false);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert!(!g.observe(p.other()).known_hand.contains(&g.object_ref(top)));
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        assert_eq!(g.observe(p).last_explore, historical);
        g.restore(&saved).unwrap();
        let wilds = named(&mut g, p, "Evolving Wilds", Zone::Battlefield);
        activate(&mut g, wilds, None);
        while !matches!(g.state.pending, Pending::EffectSearch { .. }) {
            choose(&mut g, None, 0, false);
        }
        replay(&mut g, Action::FinishSearch);
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        assert_eq!(g.observe(p).last_explore, historical);
    }
}
#[test]
fn suppressed_etb_avoids_explore_but_death_and_actual_vigilance_attack_still_work() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p.other(), "Torpor Orb", Zone::Battlefield);
        let scout = named(&mut g, p, "Cenote Scout", Zone::Hand);
        cast_stack(&mut g, scout, None, 0);
        settle(&mut g, None, false);
        assert!(g.observe(p).last_explore.is_none());
        let sentinel = named(
            &mut g,
            p,
            "Sentinel of the Nameless City",
            Zone::Battlefield,
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: sentinel });
        assert!(maps(&g, p).is_empty());
        replay(&mut g, Action::FinishAttackers);
        while !g.state.stack.is_empty() || !matches!(g.state.pending, Pending::Priority) {
            choose(&mut g, None, 0, false);
        }
        assert!(!g.object_view(sentinel).tapped);
        assert_eq!(maps(&g, p).len(), 1);
        let mut g = setup(p);
        let explorer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let wing = named(&mut g, p, "Miner's Guidewing", Zone::Battlefield);
        g.move_object(wing, Zone::Graveyard);
        settle(&mut g, Some(explorer), false);
        assert_eq!(
            g.observe(p).last_explore.unwrap().explorer.object,
            g.object_ref(explorer)
        );
        assert!(
            g.object_view(wing)
                .keywords
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
    }
}
#[test]
fn early_counter_and_late_event_or_token_capacity_failures_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Library);
        let source = named(&mut g, p, "Seeker of Sunlight", Zone::Battlefield);
        g.state.objects[source as usize].counters.plus_one = u64::MAX;
        activate(&mut g, source, None);
        replay(&mut g, Action::Pass);
        let old = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.revision(), rev);
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Library);
        let source = named(&mut g, p, "Seeker of Sunlight", Zone::Battlefield);
        activate(&mut g, source, None);
        enter_choice(&mut g);
        g.state.explore_data.as_mut().unwrap().sequence = u64::MAX;
        let object = g.observe(p).explore_choice.unwrap().revealed.object;
        let old = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(
                p,
                Action::ChooseExplore {
                    object,
                    graveyard: true
                }
            ),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(object.card).zone, Zone::Library);
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Get Lost", Zone::Hand);
        named(&mut g, p, "Eumidian Terrabotanist", Zone::Battlefield);
        cast_stack(&mut g, source, Some(victim), 0);
        replay(&mut g, Action::Pass);
        // Two vacant token slots force failure only after the first Map enters.
        let exhausted = spawn(&mut g, p, map_card(), Zone::Outside);
        let available = spawn(&mut g, p, map_card(), Zone::Outside);
        g.state.objects[exhausted as usize].generation = u64::MAX;
        g.state.free_tokens.extend([exhausted, available]);
        let old = g.snapshot();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    }
}

#[test]
fn exact_target_does_not_explore_a_reused_creature_token_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = Card::catalogue()
            .find(|c| c.is_token() && c.is_creature())
            .unwrap();
        let creature = spawn(&mut g, p, token, Zone::Battlefield);
        let old = g.object_ref(creature);
        let map = spawn(&mut g, p, map_card(), Zone::Battlefield);
        activate(&mut g, map, Some(creature));
        g.move_object(creature, Zone::Graveyard);
        g.state_based_actions();
        assert_eq!(g.object_view(creature).zone, Zone::Outside);
        g.create_tokens(p.other(), token, 1, false);
        assert_ne!(g.object_ref(creature), old);
        assert_eq!(g.object_view(creature).owner, p.other());
        settle(&mut g, None, false);
        assert!(g.observe(p).last_explore.is_none());
        assert_eq!(g.object_view(creature).counters.plus_one, 0);
    }
}
#[test]
fn seeker_real_activation_cost_and_sorcery_timing_preserve_independent_explore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Seeker of Sunlight", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [0, 0, 0, 0, 1, 0]);
        assert_eq!(request.generic, 2);
        while g.state.announcing.is_some() {
            choose(&mut g, None, 0, false);
        }
        let old = g.object_ref(source);
        g.move_object(source, Zone::Hand);
        settle(&mut g, None, false);
        assert_eq!(g.observe(p).last_explore.unwrap().explorer.object, old);
        assert_eq!(
            g.observe(p).last_explore.unwrap().destination,
            Some(Zone::Hand)
        );
        let mut g = setup(p);
        let source = named(&mut g, p, "Seeker of Sunlight", Zone::Battlefield);
        g.state.active = p.other();
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
        );
    }
}
#[test]
fn get_lost_target_union_excludes_plain_artifacts_lands_and_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let enchantment = named(&mut g, p.other(), "Turtle Power!", Zone::Battlefield);
        let walker_card = Card::catalogue()
            .find(|c| c.is_supported() && c.info().type_line.contains("Planeswalker"))
            .unwrap();
        let walker = spawn(&mut g, p.other(), walker_card, Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let artifact = named(&mut g, p.other(), "Bear Trap", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let hand = named(&mut g, p.other(), "Shock", Zone::Hand);
        let spell = named(&mut g, p, "Get Lost", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        for id in [creature, enchantment, walker] {
            assert!(g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(id)
            }));
        }
        for id in [artifact, land, hand] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
    }
}
#[test]
fn public_explore_boxes_and_index_are_fully_counted_and_restored() {
    let initial = Game::new(Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    println!(
        "initial={} ExploreState={} ExploreChoice={}",
        initial.state_bytes(),
        std::mem::size_of::<explore::ExploreState>(),
        std::mem::size_of::<ExploreChoiceView>()
    );
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Merfolk Cave-Diver", Zone::Battlefield);
    named(&mut g, Player::P0, "Bear Cub", Zone::Library);
    let source = named(&mut g, Player::P0, "Cenote Scout", Zone::Hand);
    cast_stack(&mut g, source, None, 0);
    enter_choice(&mut g);
    let bytes = g.state_bytes();
    let data = g.state.explore_data.as_ref().unwrap();
    let expected = data.bytes() + std::mem::size_of::<ExploreChoiceView>();
    let mut blank = g.clone();
    blank.state.explore_data = None;
    blank.state.resolving.as_mut().unwrap().explore = None;
    assert_eq!(bytes - blank.state_bytes(), expected);
    let saved = g.snapshot();
    let object = g
        .observe(Player::P0)
        .explore_choice
        .unwrap()
        .revealed
        .object;
    replay(
        &mut g,
        Action::ChooseExplore {
            object,
            graveyard: true,
        },
    );
    g.restore(&saved).unwrap();
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(g.state.explore_data.as_ref().unwrap().sources.len(), 1);
}
