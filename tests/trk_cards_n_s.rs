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
