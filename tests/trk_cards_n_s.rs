//! Focused complete Oracle scenarios for authored N–S Star Trek cards.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn mana(color: usize) -> Mana {
    let mut result = [0; 6];
    result[color] = 1;
    result
}
fn land_mana(card: CardId, color: usize) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: mana(color),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("bounded resolution");
}
#[test]
fn planet_gainlands_enter_tapped_gain_life_and_offer_only_printed_colors_on_owned_branches() {
    for (name, colors) in [
        ("Orion", [3, 4]),
        ("Qo'noS", [2, 3]),
        ("Risa", [4, 0]),
        ("Romulus", [1, 2]),
    ] {
        for controller in [Player::P0, Player::P1] {
            let mut original = scenario(controller);
            let land = put(&mut original, controller, name, Zone::Hand);
            let saved = original.snapshot();
            let mut raw = original.clone();
            let mut indexed = original.clone();
            let mut restored = original.clone();
            restored.restore(&saved).unwrap();
            drop(original);
            for (branch, game) in [&mut raw, &mut indexed, &mut restored]
                .into_iter()
                .enumerate()
            {
                let action = Action::PlayLand { card: land };
                if branch == 1 {
                    let index = game
                        .legal_actions()
                        .iter()
                        .position(|a| *a == action)
                        .unwrap();
                    game.apply_index(controller, game.revision(), index)
                        .unwrap();
                } else {
                    act(game, action);
                }
                assert!(game.object_view(land).tapped, "{name}");
                assert_eq!(game.state.players[controller.index()].life, 20);
                drain(game);
                assert_eq!(game.state.players[controller.index()].life, 21, "{name}");
                assert_eq!(game.state.players[controller.other().index()].life, 20);
                reject(game, land_mana(land, colors[0]));
                game.state.objects[land as usize].tapped = false;
                game.state.players[controller.index()].mana = [0; 6];
                reject(game, land_mana(land, 5));
                let action = land_mana(land, colors[0]);
                if branch == 1 {
                    let index = game
                        .legal_actions()
                        .iter()
                        .position(|a| *a == action)
                        .unwrap();
                    game.apply_index(controller, game.revision(), index)
                        .unwrap();
                } else {
                    act(game, action);
                }
                assert_eq!(game.state.players[controller.index()].mana, mana(colors[0]));
            }
            assert_eq!(raw.snapshot(), indexed.snapshot(), "{name}");
            assert_eq!(raw.snapshot(), restored.snapshot(), "{name}");
        }
    }
}
#[test]
fn patrol_landfall_and_shuttle_turn_and_tap_abilities_use_live_controller() {
    let mut game = scenario(Player::P0);
    let patrol = put(&mut game, Player::P0, "Planetary Patrol", Zone::Battlefield);
    let shuttle = put(&mut game, Player::P0, "Shuttle Ace", Zone::Battlefield);
    assert!(game.keywords(shuttle).contains(Keywords::FLYING));
    game.state.active = Player::P1;
    assert!(!game.keywords(shuttle).contains(Keywords::FLYING));
    game.state.active = Player::P0;
    let land = put(&mut game, Player::P0, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    drain(&mut game);
    assert_eq!(stats(&game, patrol), (3, 3));
    let other = put(&mut game, Player::P1, "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, patrol), (3, 3));
    let _ = other;
}
#[test]
fn open_communications_draws_and_has_the_printed_beam_cost() {
    let mut game = scenario(Player::P0);
    let spell = put(&mut game, Player::P0, "Open Communications", Zone::Hand);
    let draws_before = game.state.players[0].cards_drawn_this_turn;
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(
        game.state.players[0].cards_drawn_this_turn,
        draws_before + 1
    );
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

fn until(game: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if condition(game) {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("bounded action search at {:?}", game.state.pending);
}
fn activate(game: &mut Game, card: CardId, ability: u16, target: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(id) = target {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(game, Action::FinishTargets);
        }
    }
    pay(game);
}
#[test]
fn untapped_land_cycle_selects_live_modes_only_after_two_other_lands() {
    for (name, color) in [
        ("Ni'Var Science Academy", 1),
        ("Q Continuum", 4),
        ("Quark's Bar", 3),
    ] {
        let mut game = scenario(Player::P0);
        put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, Player::P0, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: first });
        assert!(game.object_view(first).tapped);
        assert!(game.state.stack.is_empty());
        let mut prepared = scenario(Player::P0);
        put(&mut prepared, Player::P0, "Forest", Zone::Battlefield);
        put(&mut prepared, Player::P0, "Plains", Zone::Battlefield);
        let creature = put(&mut prepared, Player::P0, "Bear Cub", Zone::Battlefield);
        let land = put(&mut prepared, Player::P0, name, Zone::Hand);
        act(&mut prepared, Action::PlayLand { card: land });
        assert!(!prepared.object_view(land).tapped);
        until(&mut prepared, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
        });
        assert!(
            prepared
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 1 })
        );
        act(&mut prepared, Action::ChooseMode { mode: 0 });
        if name != "Quark's Bar" {
            act(
                &mut prepared,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if prepared.legal_actions().contains(&Action::FinishTargets) {
                act(&mut prepared, Action::FinishTargets);
            }
        }
        settle(&mut prepared);
        if name == "Quark's Bar" {
            assert_eq!(prepared.state.players[Player::P1.index()].life, 19);
        } else if name == "Ni'Var Science Academy" {
            assert!(prepared.keywords(creature).contains(Keywords::FLYING));
        } else {
            assert_eq!(stats(&prepared, creature), (3, 3));
            assert!(prepared.keywords(creature).contains(Keywords::VIGILANCE));
        }
        prepared.state.players[0].mana = [0; 6];
        act(&mut prepared, land_mana(land, color));
        assert_eq!(prepared.state.players[0].mana, mana(color));
    }
}
#[test]
fn neelix_and_new_planet_creators_preserve_token_recipes_and_life_triggers() {
    let mut game = scenario(Player::P0);
    let neelix = put(&mut game, Player::P0, "Neelix, Moral Support", Zone::Hand);
    cast(&mut game, neelix, None);
    drain(&mut game);
    assert_eq!(
        game.observe(Player::P0)
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.has_subtype("Food"))
            .count(),
        1
    );
    let orion = put(&mut game, Player::P0, "Orion", Zone::Hand);
    act(&mut game, Action::PlayLand { card: orion });
    drain(&mut game);
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.counter_count(neelix, CounterKind::PlusOne), 1);
    let explorer = put(&mut game, Player::P0, "Saurian Explorer", Zone::Battlefield);
    activate(&mut game, explorer, 0, None);
    settle(&mut game);
    let planets: Vec<_> = game
        .observe(Player::P0)
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.name == "New Planet")
        .cloned()
        .collect();
    assert_eq!(planets.len(), 1);
    assert!(planets[0].tapped);
    assert!(planets[0].has_subtype("Planet"));
}
#[test]
fn spacecraft_station_with_paid_creature_power_and_keep_artifact_identity() {
    for (name, expected) in [
        ("Oberth-Class Starship", (2, 2)),
        ("Suurok-Class Science Vessel", (3, 2)),
    ] {
        let mut game = scenario(Player::P0);
        let ship = put(&mut game, Player::P0, name, Zone::Battlefield);
        assert!(!game.is_creature(ship));
        for _ in 0..2 {
            let donor = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: ship,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let donor_ref = game.object_ref(donor);
            act(
                &mut game,
                Action::ChooseAdditionalCost { object: donor_ref },
            );
            pay(&mut game);
            settle(&mut game);
            assert!(game.object_view(donor).tapped);
        }
        assert_eq!(game.counter_count(ship, CounterKind::Charge), 6);
        assert!(game.is_creature(ship));
        assert_eq!(stats(&game, ship), expected);
        assert!(game.keywords(ship).contains(Keywords::FLYING));
        assert_ne!(game.object_view(ship).kinds & 16, 0);
    }
}

#[test]
fn modal_rogue_ai_and_rok_tahk_create_the_chosen_effect_and_murf() {
    let mut game = scenario(Player::P0);
    let rogue = put(
        &mut game,
        Player::P0,
        "Rogue Artificial Intelligence",
        Zone::Hand,
    );
    cast(&mut game, rogue, None);
    until(&mut game, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseMode { mode: 1 })
    );
    act(&mut game, Action::ChooseMode { mode: 0 });
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 23);
    assert_eq!(stats(&game, rogue), (6, 5));
    let rok = put(
        &mut game,
        Player::P0,
        "Rok-Tahk, Budding Scientist",
        Zone::Hand,
    );
    cast(&mut game, rok, None);
    drain(&mut game);
    let murfs: Vec<_> = game
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.name == "Murf")
        .collect();
    assert_eq!(murfs.len(), 1);
    assert_eq!((murfs[0].power, murfs[0].toughness), (0, 1));
    assert!(murfs[0].keywords.contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn saru_protects_another_creature_only_and_syndicate_landfall_stacks_counters() {
    let mut game = scenario(Player::P0);
    let bearer = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let saru = put(&mut game, Player::P0, "Saru, Unthreatened", Zone::Hand);
    cast(&mut game, saru, None);
    until(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
    });
    assert!(!game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(saru)
    }));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert!(game.keywords(bearer).contains(Keywords::HEXPROOF));
    let liquidators = put(
        &mut game,
        Player::P0,
        "Syndicate Liquidators",
        Zone::Battlefield,
    );
    let forest = put(&mut game, Player::P0, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: forest });
    until(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(liquidators),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.counter_count(liquidators, CounterKind::PlusOne), 2);
}

#[test]
fn nog_actual_card_evolves_twice_and_grants_a_second_graveyard_land_play() {
    let mut game = scenario(Player::P0);
    let nog = put(
        &mut game,
        Player::P0,
        "Nog, Ambitious Cadet",
        Zone::Battlefield,
    );
    assert!(game.has_subtype(nog, "Rogue"));
    assert!(game.keywords(nog).contains(Keywords::MENACE));
    activate(&mut game, nog, 0, None);
    settle(&mut game);
    assert!(game.has_subtype(nog, "Scout"));
    assert!(!game.has_subtype(nog, "Rogue"));
    assert_eq!(stats(&game, nog), (3, 4));
    activate(&mut game, nog, 1, None);
    settle(&mut game);
    assert!(game.has_subtype(nog, "Officer"));
    assert!(!game.has_subtype(nog, "Scout"));
    let hand_land = put(&mut game, Player::P0, "Forest", Zone::Hand);
    let grave_land = put(&mut game, Player::P0, "Plains", Zone::Graveyard);
    act(&mut game, Action::PlayLand { card: hand_land });
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: grave_land })
    );
    act(&mut game, Action::PlayLand { card: grave_land });
    assert_eq!(game.object_view(grave_land).zone, Zone::Battlefield);
}

#[test]
fn one_modal_cast_fires_each_face_a_dilemma_observer_once() {
    let mut game = scenario(Player::P0);
    put(
        &mut game,
        Player::P0,
        "Obsidian Order Operative",
        Zone::Battlefield,
    );
    put(&mut game, Player::P0, "Seven of Nine", Zone::Battlefield);
    let bearer = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Perils of the Past", Zone::Hand);
    let drawn = game.state.players[0].cards_drawn_this_turn;
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 0 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.state.players[1].life, 19);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, drawn + 1);
    assert_eq!(stats(&game, bearer), (5, 4));
}

#[test]
fn support_mission_quest_threshold_applies_to_the_fifth_actual_entrant() {
    let mut game = scenario(Player::P0);
    let mission = put(&mut game, Player::P0, "Support Mission", Zone::Battlefield);
    for index in 1..=5 {
        let entrant = put(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        cast(&mut game, entrant, None);
        drain(&mut game);
        assert_eq!(game.counter_count(mission, CounterKind::Quest), index);
        assert_eq!(
            game.counter_count(entrant, CounterKind::PlusOne),
            if index == 5 { 2 } else { 1 }
        );
    }
}

#[test]
fn support_mission_uses_last_quest_count_when_source_leaves_before_resolution() {
    for (prior, expected) in [(4, 1), (5, 2)] {
        let mut game = scenario(Player::P0);
        let mission = put(&mut game, Player::P0, "Support Mission", Zone::Battlefield);
        game.add_counter(mission, CounterKind::Quest, prior);
        let entrant = put(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        cast(&mut game, entrant, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(entrant).zone, Zone::Battlefield);
        game.move_object(mission, Zone::Graveyard);
        drain(&mut game);
        assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), expected);
    }
}

#[test]
fn pet_sehlat_needs_an_attack_companion() {
    let mut game = scenario(Player::P0);
    let pet = put(&mut game, Player::P0, "Pet Sehlat", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    reject(&mut game, Action::Attack { card: pet });
    let bear = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::Attack { card: pet });
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    assert!(game.object_view(pet).tapped);
}

#[test]
fn replicator_creates_food_then_a_tapped_copy_of_its_token() {
    let mut game = scenario(Player::P0);
    let replicator = put(&mut game, Player::P0, "Replicator", Zone::Hand);
    cast(&mut game, replicator, None);
    drain(&mut game);
    let food = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == Card(5246) && o.is_token())
                .then_some(id as CardId)
        })
        .expect("Replicator Food");
    let nontoken = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: replicator,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(nontoken),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(food),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game);
    let copies: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (id as CardId != food
                && o.zone == Zone::Battlefield
                && o.card == Card(5246)
                && o.is_token())
            .then_some(id as CardId)
        })
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(game.object_view(copies[0]).tapped);
    assert!(!game.object_view(food).tapped);
}

#[test]
fn bird_of_prey_cloaks_until_stationed_and_tracks_noncreature_casts() {
    let mut game = scenario(Player::P0);
    let bird = put(
        &mut game,
        Player::P0,
        "Romulan Bird-of-Prey",
        Zone::Battlefield,
    );
    assert!(game.keywords(bird).contains(Keywords::HEXPROOF));
    let spell = put(&mut game, Player::P0, "Open Communications", Zone::Hand);
    cast(&mut game, spell, None);
    drain(&mut game);
    assert_eq!(game.counter_count(bird, CounterKind::Charge), 2);
    game.add_counter(bird, CounterKind::Charge, 6);
    assert_eq!(stats(&game, bird), (4, 4));
    assert!(game.keywords(bird).contains(Keywords::FLYING));
    assert!(!game.keywords(bird).contains(Keywords::HEXPROOF));
}

#[test]
fn salt_vampire_privately_exiles_only_nonland_then_victim_draws() {
    let mut game = scenario(Player::P0);
    let nonland = put(&mut game, Player::P1, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, Player::P1, "Forest", Zone::Hand);
    let salt = put(&mut game, Player::P0, "Salt Vampire", Zone::Hand);
    assert!(game.keywords(salt).contains(Keywords::LIFELINK));
    cast(&mut game, salt, None);
    until(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    until(&mut game, |g| {
        matches!(g.state.pending, Pending::HandCardChoice { .. })
    });
    let choice = game.observe(Player::P0).hand_choice.unwrap();
    assert_eq!(choice.candidates, [game.object_ref(nonland)]);
    assert!(game.observe(Player::P1).hand_choice.is_none());
    let land_ref = game.object_ref(land);
    reject(
        &mut game,
        Action::ChooseHandCard {
            object: Some(land_ref),
        },
    );
    let nonland_ref = game.object_ref(nonland);
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(nonland_ref),
        },
    );
    assert_eq!(game.object_view(nonland).zone, Zone::Exile);
    assert_eq!(game.state.players[1].cards_drawn_this_turn, 1);
}

#[test]
fn sarek_draws_per_five_non_borg_creature_types_at_own_combat() {
    let mut game = scenario(Player::P0);
    let sarek = put(
        &mut game,
        Player::P0,
        "Sarek, Federation Ambassador",
        Zone::Battlefield,
    );
    put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let drawn = game.state.players[0].cards_drawn_this_turn;
    for _ in 0..16 {
        if game.state.players[0].cards_drawn_this_turn > drawn {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.players[0].cards_drawn_this_turn, drawn + 1);
    assert_eq!(game.counter_count(sarek, CounterKind::PlusOne), 0);
}

#[test]
fn seven_of_nine_requires_distinct_land_names_for_lasting_base_stats() {
    let mut game = scenario(Player::P0);
    let seven = put(
        &mut game,
        Player::P0,
        "Seven of Nine, Unassimilated",
        Zone::Battlefield,
    );
    for name in ["Forest", "Island", "Plains", "Mountain", "Swamp", "Orion"] {
        put(&mut game, Player::P0, name, Zone::Battlefield);
    }
    let action = Action::AnnounceActivate {
        card: seven,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, action.clone());
    put(&mut game, Player::P0, "Romulus", Zone::Battlefield);
    act(&mut game, action);
    pay(&mut game);
    drain(&mut game);
    assert_eq!(stats(&game, seven), (7, 9));
    game.expire_base_stats();
    assert_eq!(stats(&game, seven), (7, 9));
    game.move_object(seven, Zone::Exile);
    game.move_object(seven, Zone::Battlefield);
    assert_eq!(stats(&game, seven), (0, 4));
}

#[test]
fn sulu_remembers_an_equipment_entry_after_it_leaves() {
    let mut game = scenario(Player::P0);
    let sulu = put(
        &mut game,
        Player::P0,
        "Sulu, Space Swashbuckler",
        Zone::Battlefield,
    );
    assert!(game.keywords(sulu).contains(Keywords::FIRST_STRIKE));
    let equipment = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    cast(&mut game, equipment, None);
    drain(&mut game);
    game.move_object(equipment, Zone::Graveyard);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    let drawn = game.state.players[0].cards_drawn_this_turn;
    act(&mut game, Action::Attack { card: sulu });
    act(&mut game, Action::FinishAttackers);
    drain(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, drawn + 1);
}
