use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
use crate::{SearchDestination, SearchFilter};
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision;
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision, revision);
}
fn settle(g: &mut Game, target: Option<Target>) {
    for _ in 0..100 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let actions = g.legal_actions();
        let a = target
            .and_then(|t| {
                actions
                    .iter()
                    .find(|a| matches!(a,Action::ChooseTarget{target}if *target==t))
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::Pass
                            | Action::Discard { .. }
                            | Action::ChooseOptional { accept: false }
                            | Action::FinishTargets
                    )
                })
            })
            .unwrap()
            .clone();
        replay(g, a);
    }
    panic!("fixture didn't settle");
}
fn cast_funded(g: &mut Game, name: &str) -> CardId {
    let c = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(g, c, None);
    c
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| s.card == id)
        .count()
}
// Synthetic instruction continuations, never additional printed-card admissions.
fn program(g: &mut Game, code: &'static [Instruction]) {
    let id = spawn(g, Player::P0, Card::Shock, Zone::Hand);
    let item = g.cast_item(Player::P0, id, [0; 6]);
    g.state.resolving = Some(Resolution {
        wheel: None,
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
        mana_payment: None,
        forage: None,
    });
    g.run_program();
}

#[test]
fn changeling_has_every_pinned_creature_type_and_no_unprinted_other_subtypes_in_all_zones() {
    assert_eq!(crate::CREATURE_TYPES.len(), 324);
    let c = Card::from_name("Chitinous Graspling").unwrap();
    assert!(c.keywords().contains(Keywords::CHANGELING));
    for typ in crate::CREATURE_TYPES {
        assert!(c.has_subtype(typ), "{typ}");
        assert!(crate::is_creature_type(typ));
    }
    for typ in [
        "Forest",
        "Plains",
        "Equipment",
        "Vehicle",
        "Aura",
        "Food",
        "Treasure",
        "Saga",
        "Time",
        "Lord",
        "Elf Warrior",
        "Klingon",
        "NotAType",
        "",
    ] {
        assert!(!c.has_subtype(typ), "{typ}");
    }
    assert!(!c.has_subtype("elf"));
    assert!(c.has_subtype("Time Lord"));
    assert!(c.has_subtype("Assembly-Worker"));
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Library);
    for zone in [
        Zone::Library,
        Zone::Hand,
        Zone::Stack,
        Zone::Battlefield,
        Zone::Graveyard,
        Zone::Exile,
        Zone::Outside,
    ] {
        if g.object_view(id).zone != zone {
            g.move_object(id, zone);
        }
        assert!(g.object_view(id).card.has_subtype("Elf"));
        assert!(g.object_view(id).keywords.contains(Keywords::CHANGELING));
    }
}

#[test]
fn ordinary_subtypes_and_token_types_remain_exact() {
    assert!(Card::Forest.has_subtype("Forest"));
    assert!(!Card::Forest.has_subtype("Elf"));
    let bear = Card::from_name("Bear Cub").unwrap();
    assert!(bear.has_subtype("Bear"));
    assert!(!bear.has_subtype("Elf"));
    let food = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Food"))
        .unwrap();
    assert!(food.has_subtype("Food"));
    assert!(!food.has_subtype("Bear"));
    let robot = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Robot"))
        .unwrap();
    assert!(robot.has_subtype("Robot"));
    assert!(!robot.has_subtype("Equipment"));
    let c = Card::from_name("Chitinous Graspling").unwrap();
    assert!(!SearchFilter::Subtype("Forest").matches(c));
    assert!(!SearchFilter::BasicLand.matches(c));
    assert!(SearchFilter::Subtype("Elf").matches(c));
}

#[test]
fn changeling_receives_owned_elf_lord_bonuses_and_restores_without_a_type_cache() {
    let mut g = scenario();
    let lord = named(&mut g, Player::P0, "Imperious Perfect", Zone::Battlefield);
    let own = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Battlefield);
    let other = named(&mut g, Player::P1, "Chitinous Graspling", Zone::Battlefield);
    let hand = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Hand);
    let base = g.state.objects[own as usize].local_power();
    assert_eq!(g.stats(own).power, base + 1);
    assert_eq!(g.stats(other).power, base);
    assert_eq!(g.stats(hand).power, base);
    let saved = g.snapshot();
    let size = g.state_bytes();
    g.move_object(lord, Zone::Graveyard);
    assert_eq!(g.stats(own).power, base);
    g.restore(&saved).unwrap();
    assert_eq!(g.stats(own).power, base + 1);
    assert_eq!(g.state_bytes(), size);
    assert!(g.object_view(own).card.has_subtype("Dinosaur"));
    assert!(g.object_view(own).card.has_subtype("Turtle"));
}

#[test]
fn a_changeling_entry_matches_multiple_subtype_observers_with_owner_and_other_rules() {
    let mut g = scenario();
    let mut sources = Vec::new();
    for name in [
        "Mary Jane Watson",
        "Avatar Enthusiasts",
        "Youthful Valkyrie",
        "Obyra, Dreaming Duelist",
        "Flying Octobot",
    ] {
        sources.push(named(&mut g, Player::P0, name, Zone::Battlefield));
        named(&mut g, Player::P1, name, Zone::Battlefield);
    }
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Hand);
    g.move_object(c, Zone::Battlefield);
    for &id in &sources {
        assert_eq!(count(&g, id), 1, "{}", g.object_view(id).card.name());
    }
    assert_eq!(g.state.pending_triggers.len(), sources.len());
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.state.players[1].life, 19);
    for &id in &[sources[1], sources[2], sources[4]] {
        assert_eq!(g.object_view(id).counters.plus_one, 1);
    }
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Hand);
    g.move_object(c, Zone::Battlefield);
    assert_eq!(count(&g, sources[0]), 0);
    assert_eq!(count(&g, sources[4]), 0);
    assert_eq!(count(&g, sources[1]), 1);
}

#[test]
fn changeling_death_uses_lki_subtypes_even_if_observer_dies_in_same_batch() {
    let mut g = scenario();
    let boggart = named(
        &mut g,
        Player::P0,
        "Boggart Cursecrafter",
        Zone::Battlefield,
    );
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Chitinous Graspling", Zone::Battlefield);
    g.move_object(enemy, Zone::Graveyard);
    assert_eq!(count(&g, boggart), 0);
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[c, boggart], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    assert_eq!(count(&g, boggart), 1);
    settle(&mut g, None);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn captured_damage_dealer_remains_a_hero_and_ally_after_departure() {
    let mut g = scenario();
    let thing = named(
        &mut g,
        Player::P0,
        "The Thing, Ben Grimm",
        Zone::Battlefield,
    );
    let tactics = named(&mut g, Player::P0, "Invasion Tactics", Zone::Battlefield);
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Battlefield);
    let old = g.damage_source(c);
    g.move_object(c, Zone::Graveyard);
    g.player_damage_triggers(&[DamageEvent {
        preventable: true,
        source: old,
        target: Target::Player(Player::P1),
        amount: 1,
        combat: true,
    }]);
    assert_eq!(count(&g, thing), 1);
    assert_eq!(count(&g, tactics), 1);
    settle(&mut g, None);
    assert_eq!(g.object_view(thing).counters.plus_one, 2);
    assert_eq!(g.state.players[0].library.len(), 19);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.player_damage_triggers(&[DamageEvent {
        preventable: true,
        source: g.damage_source(bear),
        target: Target::Player(Player::P1),
        amount: 2,
        combat: true,
    }]);
    assert_eq!(count(&g, thing), 0);
}

#[test]
fn subtype_library_search_selects_changeling_privately_but_land_search_does_not() {
    const CODE: &[Instruction] = &[Instruction::Search(SearchSpec {
        filter: SearchFilter::Subtype("Elf"),
        maximum: 1,
        destination: SearchDestination::Hand,
        tapped: false,
        reveal: true,
    })];
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Library);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Library);
    program(&mut g, CODE);
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: c })
    );
    reject(&mut g, Action::ChooseSearch { card: bear });
    assert!(g.observe(Player::P1).search_choice.is_none());
    replay(&mut g, Action::ChooseSearch { card: c });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(c).zone, Zone::Hand);
    assert!(g.observe(Player::P1).known_hand.contains(&g.object_ref(c)));
}

#[test]
fn subtype_sacrifice_choice_accepts_owned_changeling_and_rejects_unrelated_cards() {
    const CODE: &[Instruction] = &[Instruction::ChooseObjects {
        kind: EffectChoiceKind::Sacrifice,
        players: PlayerSet::Controller,
        filter: SacrificeFilter {
            artifact_enchantment_or_token: false,
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 1,
            nonland: false,
            other: false,
            tokens: None,
            subtype: Some("Elf"),
        },
        amount: 1,
        optional_skip: None,
    }];
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Chitinous Graspling", Zone::Battlefield);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Chitinous Graspling", Zone::Battlefield);
    program(&mut g, CODE);
    let bear_ref = g.object_ref(bear);
    reject(&mut g, Action::Sacrifice { object: bear_ref });
    let enemy_ref = g.object_ref(enemy);
    reject(&mut g, Action::Sacrifice { object: enemy_ref });
    let object = g.object_ref(c);
    replay(&mut g, Action::Sacrifice { object });
    assert_eq!(g.object_view(c).zone, Zone::Graveyard);
}

#[test]
fn all_new_keyword_only_bodies_keep_their_complete_keywords_and_hybrid_costs() {
    for (name, extra) in [
        ("Chitinous Graspling", Keywords::REACH),
        ("Gangly Stompling", Keywords::TRAMPLE),
        ("Mischievous Sneakling", Keywords::FLASH),
        ("Prideful Feastling", Keywords::LIFELINK),
    ] {
        let mut g = scenario();
        let id = cast_funded(&mut g, name);
        settle(&mut g, None);
        assert!(
            g.object_view(id)
                .keywords
                .contains(Keywords::CHANGELING.union(extra))
        );
        assert!(g.object_view(id).card.has_subtype("Elf"));
    }
}

#[test]
fn optional_basic_search_may_decline_without_shuffling_or_accept_private_search() {
    for name in ["Changeling Wayfinder", "Environmental Scientist"] {
        for accept in [false, true] {
            let mut g = scenario();
            cast_funded(&mut g, name);
            pass_pair(&mut g);
            pass_pair(&mut g);
            let rng = g.state.rng;
            let original = g.state.players[0].library.clone();
            assert!(matches!(g.state.pending, Pending::EffectOptional { .. }));
            replay(&mut g, Action::ChooseOptional { accept });
            if accept {
                let card = g.state.players[0].library[0];
                replay(&mut g, Action::ChooseSearch { card });
                replay(&mut g, Action::FinishSearch);
                assert_eq!(g.object_view(card).zone, Zone::Hand);
                assert_ne!(g.state.rng, rng);
            } else {
                assert_eq!(g.state.rng, rng);
                assert_eq!(g.state.players[0].library, original);
            }
        }
    }
}

#[test]
fn chomping_optional_removal_omits_or_destroys_legal_targets_and_farm_animal_sacrifice_triggers() {
    for take in [false, true] {
        let mut g = scenario();
        let teddy = named(&mut g, Player::P1, "Friendly Teddy", Zone::Battlefield);
        let c = cast_funded(&mut g, "Chomping Changeling");
        pass_pair(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(c),
            },
        );
        if take {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(teddy),
                },
            );
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        settle(&mut g, None);
        assert_eq!(
            g.object_view(teddy).zone,
            if take {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
    let mut g = scenario();
    let farm = cast_funded(&mut g, "Curious Farm Animals");
    settle(&mut g, None);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..}if *card==farm))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(farm).zone, Zone::Graveyard);
    settle(&mut g, None);
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn flock_impostor_optional_return_preserves_other_and_owner_constraints() {
    for take in [false, true] {
        let mut g = scenario();
        let own = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let foe = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        let bird = cast_funded(&mut g, "Flock Impostor");
        pass_pair(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bird),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        if take {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        settle(&mut g, None);
        assert_eq!(
            g.object_view(own).zone,
            if take { Zone::Hand } else { Zone::Battlefield }
        );
        assert!(
            g.keywords(bird).contains(
                Keywords::FLYING
                    .union(Keywords::FLASH)
                    .union(Keywords::CHANGELING)
            )
        );
    }
}

#[test]
fn graveshifter_targets_are_required_even_when_return_effect_is_optional() {
    for accept in [false, true] {
        let mut g = scenario();
        let own = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Graveyard);
        let foe = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Graveyard);
        cast_funded(&mut g, "Graveshifter");
        pass_pair(&mut g);
        reject(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pass_pair(&mut g);
        replay(&mut g, Action::ChooseOptional { accept });
        assert_eq!(
            g.object_view(own).zone,
            if accept { Zone::Hand } else { Zone::Graveyard }
        );
    }
}

#[test]
fn rooftop_optional_multiple_graveyard_targets_do_not_suppress_independent_life_gain() {
    for number in 0..=2 {
        let mut g = scenario();
        let a = spawn(&mut g, Player::P0, Card::Forest, Zone::Graveyard);
        let b = spawn(&mut g, Player::P1, Card::Forest, Zone::Graveyard);
        cast_funded(&mut g, "Rooftop Percher");
        pass_pair(&mut g);
        if number > 0 {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
        }
        if number > 1 {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        settle(&mut g, None);
        assert_eq!(g.state.players[0].life, 23);
        assert_eq!(
            g.object_view(a).zone,
            if number > 0 {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            g.object_view(b).zone,
            if number > 1 {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn three_tree_mascot_conversion_is_once_per_turn_not_one_mana_per_color() {
    let mut g = scenario();
    let c = cast_funded(&mut g, "Three Tree Mascot");
    settle(&mut g, None);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    let actions = g
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a,Action::ActivateMana{card,..}if *card==c))
        .collect::<Vec<_>>();
    assert_eq!(actions.len(), 5);
    replay(&mut g, actions[0].clone());
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..}if *card==c)
    ));
    g.state.active = Player::P1;
    g.begin_turn();
    g.state.priority = Player::P0;
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana{card,..}if *card==c))
    );
}
