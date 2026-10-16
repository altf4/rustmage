use super::tests::{act, scenario, spawn};
use super::*;
const FIXED: [&str; 5] = [
    "Chrome Companion",
    "Hoverstone Pilgrim",
    "Jade-Cast Sentinel",
    "Malevolent Chandelier",
    "The Spot's Portal",
];
const CHOICES: [&str; 8] = [
    "Lost in Space",
    "Misleading Motes",
    "Return to the Sewers",
    "Riverwalk Technique",
    "Trip Up",
    "Uncharted Voyage",
    "Unlucky Drop",
    "Vanish from Sight",
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
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn commit_destination(g: &mut Game, id: CardId, destination: LibraryDestination) {
    let object = g.object_ref(id);
    replay(
        g,
        Action::ChooseLibraryDestination {
            object,
            destination,
        },
    );
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn choose(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::PutTrigger { .. }
                    | Action::ChooseMode { mode: 0 }
                    | Action::ChooseOptional { accept: true }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
                    | Action::ChooseLibraryDestination {
                        destination: LibraryDestination::Top,
                        ..
                    }
                    | Action::Pass
            )
        })
        .expect("continuation");
    replay(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled");
}
fn announce(g: &mut Game, id: CardId, target: Option<Target>, mode: u16) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        if matches!(g.state.pending, Pending::Modes { .. }) {
            replay(g, Action::ChooseMode { mode });
        } else if matches!(g.state.pending, Pending::Targets { .. }) {
            replay(
                g,
                Action::ChooseTarget {
                    target: target.unwrap(),
                },
            );
        } else {
            choose(g);
        }
    }
}
fn window(g: &mut Game, name: &str, victim: CardId) -> CardId {
    let p = g.state.active;
    let s = named(g, p, name, Zone::Hand);
    announce(g, s, Some(Target::Card(victim)), 0);
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::TargetLibrary { .. }) {
            return s;
        }
        choose(g);
    }
    panic!("no destination");
}
#[test]
fn all_thirteen_complete_bodies_use_owner_libraries_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for name in FIXED.into_iter().chain(CHOICES) {
            for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
                let mut g = setup(p);
                let q = p.other();
                let grave = FIXED[..4].contains(&name);
                let victim = named(
                    &mut g,
                    q,
                    "Bear Cub",
                    if grave {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    },
                );
                let r = g.object_ref(victim);
                let old = g.state.players[q.index()].library.clone();
                let source = named(&mut g, p, name, Zone::Hand);
                announce(
                    &mut g,
                    source,
                    if grave {
                        None
                    } else {
                        Some(Target::Card(victim))
                    },
                    0,
                );
                if grave {
                    settle(&mut g);
                    g.state.objects[source as usize].summoning_sick = false;
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
                            target: Target::Card(victim),
                        },
                    );
                    while g.state.announcing.is_some() {
                        choose(&mut g);
                    }
                }
                if CHOICES.contains(&name) {
                    while !matches!(g.state.pending, Pending::TargetLibrary { .. }) {
                        choose(&mut g);
                    }
                    assert_eq!(g.actor(), Some(q));
                    assert_eq!(g.object_ref(victim), r);
                    assert_eq!(g.state.players[q.index()].library, old);
                    let v = g.observe(q).target_library.unwrap();
                    assert_eq!(
                        (v.player, v.target, v.source),
                        (q, r, g.state.resolving.as_ref().unwrap().item.object)
                    );
                    assert!(g.observe(p).target_library.is_none());
                    assert!(g.observe(p).actions.is_empty());
                    replay(
                        &mut g,
                        Action::ChooseLibraryDestination {
                            object: r,
                            destination,
                        },
                    );
                }
                settle(&mut g);
                let library = &g.state.players[q.index()].library;
                let expected = if CHOICES.contains(&name) && destination == LibraryDestination::Top
                {
                    library.last()
                } else {
                    library.first()
                };
                assert_eq!(expected, Some(&victim), "{name}");
                assert_eq!(g.object_ref(victim).generation, r.generation + 1);
                assert_eq!(g.object_view(victim).zone, Zone::Library);
                assert!(!g.observe(q).objects.iter().any(|v| v.id == victim));
                assert!(!g.observe(p).objects.iter().any(|v| v.id == victim));
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
                if name == "Chrome Companion" {
                    assert_eq!(g.state.players[p.index()].life, 21);
                }
                if name == "The Spot's Portal" {
                    assert_eq!(g.state.players[p.index()].life, 18);
                }
                if name == "Return to the Sewers" {
                    assert_eq!(
                        g.observe(p)
                            .objects
                            .iter()
                            .filter(|v| v.owner == p
                                && v.zone == Zone::Battlefield
                                && v.card.has_subtype("Mutagen"))
                            .count(),
                        1
                    );
                }
            }
        }
    }
}
#[test]
fn forged_objects_wrong_actors_destinations_and_stale_indices_are_atomic() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    window(&mut g, "Misleading Motes", id);
    let r = g.object_ref(id);
    let s = g.snapshot();
    assert_eq!(
        g.apply(
            Player::P0,
            Action::ChooseLibraryDestination {
                object: r,
                destination: LibraryDestination::Top
            }
        ),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), s);
    reject(
        &mut g,
        Action::ChooseLibraryDestination {
            object: ObjectRef {
                generation: r.generation + 1,
                ..r
            },
            destination: LibraryDestination::Top,
        },
    );
    reject(
        &mut g,
        Action::ChooseLibraryDestination {
            object: r,
            destination: LibraryDestination::Graveyard,
        },
    );
    reject(
        &mut g,
        Action::ChooseLibraryDestination {
            object: ObjectRef {
                card: CardId::MAX,
                generation: 0,
            },
            destination: LibraryDestination::Bottom,
        },
    );
    replay(
        &mut g,
        Action::ChooseLibraryDestination {
            object: r,
            destination: LibraryDestination::Top,
        },
    );
}
#[test]
fn divergent_owned_destinations_preserve_untouched_library_and_incarnations() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    window(&mut g, "Misleading Motes", id);
    let r = g.object_ref(id);
    let s = g.snapshot();
    let old = g.state.players[1].library.clone();
    let mut f = g.clone();
    act(
        &mut f,
        Action::ChooseLibraryDestination {
            object: r,
            destination: LibraryDestination::Bottom,
        },
    );
    assert_eq!(g.snapshot(), s);
    act(
        &mut g,
        Action::ChooseLibraryDestination {
            object: r,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(&g.state.players[1].library[..old.len()], old.as_slice());
    assert_eq!(&f.state.players[1].library[1..], old.as_slice());
    assert_eq!(g.object_ref(id), f.object_ref(id));
    assert_ne!(g.snapshot(), f.snapshot());
}
#[test]
fn following_surveil_and_mutagen_wait_for_owner_choice_then_return_to_controller() {
    for name in [
        "Lost in Space",
        "Uncharted Voyage",
        "Vanish from Sight",
        "Return to the Sewers",
    ] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        window(&mut g, name, id);
        assert!(g.observe(Player::P0).library_choice.is_none());
        assert!(
            !g.observe(Player::P0)
                .objects
                .iter()
                .any(|v| v.card.has_subtype("Mutagen"))
        );
        commit_destination(&mut g, id, LibraryDestination::Bottom);
        assert_eq!(g.object_view(id).zone, Zone::Library);
        if name != "Return to the Sewers" {
            assert_eq!(g.actor(), Some(Player::P0));
            assert!(g.observe(Player::P0).library_choice.is_some());
            assert!(g.observe(Player::P1).library_choice.is_none());
        }
        settle(&mut g);
    }
}
#[test]
fn target_loss_reentry_and_hexproof_fizzle_before_any_destination_or_following_effect() {
    for variant in 0..3 {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let s = named(&mut g, Player::P0, "Lost in Space", Zone::Hand);
        announce(&mut g, s, Some(Target::Card(id)), 0);
        if variant == 2 {
            g.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
        } else {
            g.move_object(id, Zone::Graveyard);
            if variant == 1 {
                g.move_object(id, Zone::Battlefield);
            }
        }
        settle(&mut g);
        assert!(g.state.resolving.is_none());
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert_ne!(g.object_view(id).zone, Zone::Library);
    }
}
#[test]
fn pending_destination_never_moves_a_departed_and_reentered_target() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    window(&mut g, "Misleading Motes", id);
    let r = g.object_ref(id);
    g.move_object(id, Zone::Graveyard);
    g.move_object(id, Zone::Battlefield);
    replay(
        &mut g,
        Action::ChooseLibraryDestination {
            object: r,
            destination: LibraryDestination::Bottom,
        },
    );
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    assert_ne!(g.object_ref(id), r);
}
#[test]
fn early_target_and_late_spell_departure_capacity_failures_restore_the_entire_choice() {
    for late in [false, true] {
        let mut g = setup(Player::P0);
        let previous = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        window(&mut g, "Misleading Motes", previous);
        commit_destination(&mut g, previous, LibraryDestination::Bottom);
        assert!(g.observe(Player::P0).last_target_library_move.is_some());
        let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        if !late {
            g.state.objects[id as usize].generation = u64::MAX;
        }
        let source = window(&mut g, "Misleading Motes", id);
        if late {
            g.state.objects[source as usize].generation = u64::MAX;
            g.state.resolving.as_mut().unwrap().item.object = g.object_ref(source);
        }
        let s = g.snapshot();
        let rev = g.revision();
        let r = g.object_ref(id);
        assert!(matches!(
            g.apply(
                Player::P1,
                Action::ChooseLibraryDestination {
                    object: r,
                    destination: LibraryDestination::Bottom
                }
            ),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(g.observe(Player::P1).target_library.is_some());
    }
}
#[test]
fn independent_grave_activation_survives_source_departure_and_rechecks_targets() {
    for name in FIXED[..4].iter() {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P1, "Opt", Zone::Graveyard);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
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
                target: Target::Card(id),
            },
        );
        while g.state.announcing.is_some() {
            choose(&mut g);
        }
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[1].library[0], id);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}
#[test]
fn riverwalk_retains_its_noncreature_counter_mode_and_rejects_creature_spells() {
    let mut g = setup(Player::P0);
    let opt = named(&mut g, Player::P0, "Opt", Zone::Hand);
    announce(&mut g, opt, Some(Target::Player(Player::P0)), 0);
    replay(&mut g, Action::Pass);
    let river = named(&mut g, Player::P1, "Riverwalk Technique", Zone::Hand);
    announce(&mut g, river, Some(Target::Spell(opt)), 1);
    settle(&mut g);
    assert_eq!(g.object_view(opt).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    let mut g = setup(Player::P0);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    announce(&mut g, bear, None, 0);
    replay(&mut g, Action::Pass);
    let river = named(&mut g, Player::P1, "Riverwalk Technique", Zone::Hand);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: river })
    );
}
#[test]
fn trip_up_cycling_and_mutagen_activation_are_complete() {
    let mut g = setup(Player::P0);
    let trip = named(&mut g, Player::P0, "Trip Up", Zone::Hand);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: trip,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        choose(&mut g);
    }
    settle(&mut g);
    assert_eq!(g.object_view(trip).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    window(&mut g, "Return to the Sewers", target);
    commit_destination(&mut g, target, LibraryDestination::Top);
    settle(&mut g);
    let token = g
        .observe(Player::P0)
        .objects
        .iter()
        .find(|v| v.card.has_subtype("Mutagen"))
        .unwrap()
        .id;
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: token,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    while g.state.announcing.is_some() {
        choose(&mut g);
    }
    settle(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert_eq!(g.object_view(token).zone, Zone::Outside);
}
#[test]
fn spot_condition_uses_live_own_villains_after_the_target_moves() {
    for live in [false, true] {
        let mut g = setup(Player::P0);
        let id = named(
            &mut g,
            Player::P0,
            "Morbius the Living Vampire",
            Zone::Battlefield,
        );
        if live {
            named(&mut g, Player::P0, "Chitinous Graspling", Zone::Battlefield);
        }
        let s = named(&mut g, Player::P0, "The Spot's Portal", Zone::Hand);
        announce(&mut g, s, Some(Target::Card(id)), 0);
        settle(&mut g);
        assert_eq!(g.state.players[0].life, if live { 20 } else { 18 });
    }
}
#[test]
fn tuck_uses_attachment_combat_cleanup_and_tokens_cease_to_exist() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let equipment = named(&mut g, Player::P1, "Goldvein Pick", Zone::Battlefield);
    g.attach_equipment(g.object_ref(equipment), g.object_ref(id));
    g.state.attackers.push(Attacker {
        card: id,
        defender: Some(CombatDefender::Player(Player::P0)),
        blocked: false,
    });
    window(&mut g, "Misleading Motes", id);
    commit_destination(&mut g, id, LibraryDestination::Bottom);
    assert!(g.state.attackers.is_empty());
    assert!(g.object_view(equipment).attached_to.is_none());
    let token = Card::catalogue()
        .find(|c| c.is_token() && c.is_creature())
        .unwrap();
    let id = spawn(&mut g, Player::P1, token, Zone::Battlefield);
    window(&mut g, "Misleading Motes", id);
    commit_destination(&mut g, id, LibraryDestination::Top);
    settle(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Outside);
    assert!(!g.state.players[1].library.contains(&id));
}

#[test]
fn full_activation_timing_and_ward_costs_remain_enforced() {
    for name in ["Chrome Companion", "Jade-Cast Sentinel"] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        named(&mut g, Player::P1, "Opt", Zone::Graveyard);
        g.state.objects[source as usize].summoning_sick = true;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==source))
        );
        g.state.objects[source as usize].summoning_sick = false;
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==source))
        );
    }
    let mut g = setup(Player::P0);
    let source = named(
        &mut g,
        Player::P0,
        "Malevolent Chandelier",
        Zone::Battlefield,
    );
    named(&mut g, Player::P1, "Opt", Zone::Graveyard);
    g.state.active = Player::P1;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==source))
    );
    for pay in [false, true] {
        let mut g = setup(Player::P0);
        let victim = named(&mut g, Player::P1, "Hoverstone Pilgrim", Zone::Battlefield);
        let source = named(&mut g, Player::P0, "Misleading Motes", Zone::Hand);
        announce(&mut g, source, Some(Target::Card(victim)), 0);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            choose(&mut g);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        let action = if pay {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                .unwrap()
        } else {
            Action::DeclineResolutionMana
        };
        replay(&mut g, action);
        settle(&mut g);
        assert_eq!(
            g.object_view(victim).zone,
            if pay {
                Zone::Library
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn fixed_bottom_failure_and_later_life_loss_overflow_restore_the_resolving_action() {
    for tail in [false, true] {
        let mut g = setup(Player::P0);
        let victim = named(
            &mut g,
            Player::P1,
            "Bear Cub",
            if tail {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            },
        );
        if tail {
            g.state.players[0].life_lost_this_turn = u128::MAX;
        } else {
            g.state.objects[victim as usize].generation = u64::MAX;
        }
        let source = named(
            &mut g,
            Player::P0,
            if tail {
                "The Spot's Portal"
            } else {
                "Hoverstone Pilgrim"
            },
            if tail { Zone::Hand } else { Zone::Battlefield },
        );
        if tail {
            announce(&mut g, source, Some(Target::Card(victim)), 0);
        } else {
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
                    target: Target::Card(victim),
                },
            );
            while g.state.announcing.is_some() {
                choose(&mut g);
            }
        }
        replay(&mut g, Action::Pass);
        let snapshot = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(Player::P1, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.revision(), revision);
        assert_eq!(
            g.object_view(victim).zone,
            if tail {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn public_destination_events_distinguish_both_branches_and_replay_for_both_viewers() {
    let mut g = setup(Player::P0);
    let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let source = window(&mut g, "Misleading Motes", victim);
    let before = g.object_ref(victim);
    let source_ref = g.object_ref(source);
    let mut fork = g.clone();
    commit_destination(&mut g, victim, LibraryDestination::Top);
    commit_destination(&mut fork, victim, LibraryDestination::Bottom);
    for viewer in [Player::P0, Player::P1] {
        let a = g.observe(viewer).last_target_library_move.unwrap();
        let b = fork.observe(viewer).last_target_library_move.unwrap();
        assert_eq!(
            (a.source, a.card, a.target_before, a.target_after, a.owner),
            (
                source_ref,
                Card::from_name("Bear Cub").unwrap(),
                before,
                g.object_ref(victim),
                Player::P1
            )
        );
        assert_eq!(
            (a.destination, b.destination),
            (LibraryDestination::Top, LibraryDestination::Bottom)
        );
        assert_ne!(a, b);
        assert!(
            g.observe(viewer)
                .objects
                .iter()
                .all(|v| v.zone != Zone::Library)
        );
    }
    assert_eq!(
        g.observe(Player::P0).last_target_library_move,
        g.observe(Player::P1).last_target_library_move
    );
}
#[test]
fn public_move_is_historical_through_private_reorder_draw_and_later_replacement() {
    for reorder in [false, true] {
        let mut g = setup(Player::P0);
        let victim = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        window(&mut g, "Misleading Motes", victim);
        commit_destination(&mut g, victim, LibraryDestination::Top);
        let event = g.observe(Player::P1).last_target_library_move.unwrap();
        if reorder {
            let opt = named(&mut g, Player::P0, "Opt", Zone::Hand);
            announce(&mut g, opt, None, 0);
            while !matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                choose(&mut g);
            }
            assert!(g.observe(Player::P1).library_choice.is_none());
            assert_eq!(g.observe(Player::P1).last_target_library_move, Some(event));
            replay(
                &mut g,
                Action::ArrangeLibrary {
                    card: victim,
                    destination: LibraryDestination::Bottom,
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Library);
        } else {
            g.draw(Player::P0);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert_ne!(g.object_ref(victim), event.target_after);
            // Its publicly known top identity is retained through the observed draw.
            assert!(
                g.observe(Player::P1)
                    .known_hand
                    .contains(&g.object_ref(victim))
            );
        }
        for viewer in [Player::P0, Player::P1] {
            assert_eq!(g.observe(viewer).last_target_library_move, Some(event));
        }
        let next = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        window(&mut g, "Misleading Motes", next);
        commit_destination(&mut g, next, LibraryDestination::Bottom);
        assert_ne!(g.observe(Player::P1).last_target_library_move, Some(event));
        assert_eq!(
            g.observe(Player::P1)
                .last_target_library_move
                .unwrap()
                .target_before
                .card,
            next
        );
    }
}
#[test]
fn own_top_move_is_visible_before_following_surveil_and_uses_the_new_incarnation() {
    let mut g = setup(Player::P0);
    let victim = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let before = g.object_ref(victim);
    window(&mut g, "Lost in Space", victim);
    commit_destination(&mut g, victim, LibraryDestination::Top);
    let view = g.observe(Player::P0);
    let event = view.last_target_library_move.unwrap();
    assert_eq!(event.target_before, before);
    assert_eq!(event.target_after.generation, before.generation + 1);
    assert_eq!(
        view.library_choice.unwrap().remaining[0].reference,
        event.target_after
    );
    let public = g.observe(Player::P1);
    assert!(public.library_choice.is_none());
    assert_eq!(public.last_target_library_move, Some(event));
    settle(&mut g);
}
#[test]
fn token_top_or_bottom_then_surveil_only_looks_at_real_cards_before_sbas() {
    for empty in [false, true] {
        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let mut g = setup(Player::P0);
            if empty {
                let ids = g.state.players[0].library.clone();
                for id in ids.iter().copied() {
                    g.move_object(id, Zone::Exile);
                }
            }
            let actual = g.state.players[0].library.last().copied();
            let factory = Card::catalogue()
                .find(|c| c.is_token() && c.is_creature())
                .unwrap();
            let victim = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
            window(&mut g, "Lost in Space", victim);
            commit_destination(&mut g, victim, destination);
            assert!(!g.state.players[0].library.contains(&victim));
            let event = g.observe(Player::P1).last_target_library_move.unwrap();
            assert_eq!(event.card, factory);
            if let Some(id) = actual {
                // No premature SBA: token remains in Library while a different real
                // card is privately surveilled; it cannot be looked at or moved again.
                assert_eq!(g.object_view(victim).zone, Zone::Library);
                let view = g.observe(Player::P0).library_choice.unwrap();
                assert_eq!(view.remaining[0].id, id);
                assert!(!view.remaining.iter().any(|v| v.id == victim));
                assert!(!g.move_object(victim, Zone::Hand).moved);
            }
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Outside);
            assert_eq!(g.observe(Player::P1).last_target_library_move, Some(event));
        }
    }
}

#[test]
fn token_library_entry_cannot_be_drawn_even_before_the_normal_sba_boundary() {
    static PROGRAM: &[Instruction] = &[
        Instruction::OwnerLibrary,
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    for empty in [false, true] {
        let mut g = setup(Player::P0);
        if empty {
            let ids = g.state.players[0].library.clone();
            for id in ids.iter().copied() {
                g.move_object(id, Zone::Exile);
            }
        }
        let actual = g.state.players[0].library.last().copied();
        let factory = Card::catalogue()
            .find(|c| c.is_token() && c.is_creature())
            .unwrap();
        let victim = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
        window(&mut g, "Lost in Space", victim);
        // Exercise the common instruction boundary without admitting an extra
        // card body; this draw occurs in the same resolution as the token move.
        g.state.resolving.as_mut().unwrap().program = PROGRAM;
        commit_destination(&mut g, victim, LibraryDestination::Top);
        assert_eq!(g.object_view(victim).zone, Zone::Outside);
        assert!(!g.state.players[0].library.contains(&victim));
        if let Some(id) = actual {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
            assert!(!g.observe(Player::P1).objects.iter().any(|v| v.id == id));
            assert_eq!(g.outcome(), None);
        } else {
            assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
            assert_eq!(g.outcome(), Some(Outcome::Winner(Player::P1)));
        }
    }
}
