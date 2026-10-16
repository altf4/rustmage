use super::tests::{act, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}
fn enter(game: &mut Game, owner: Player, name: &str) -> CardId {
    let id = named(game, owner, name, Zone::Hand);
    game.move_object(id, Zone::Battlefield);
    id
}
fn mana_actions(game: &Game, id: CardId) -> Vec<Action> {
    game.legal_actions()
        .into_iter()
        .filter(|a| {
            matches!(a,
        Action::ActivateMana { card, .. } if *card == id)
        })
        .collect()
}

#[test]
fn land_play_is_not_a_spell_tapped_sources_cannot_fund_and_untap_replays() {
    let mut game = scenario();
    let land = named(&mut game, Player::P0, "Azorius Guildgate", Zone::Hand);
    let second = named(&mut game, Player::P0, "Plains", Zone::Hand);
    let lions = named(&mut game, Player::P0, "Savannah Lions", Zone::Hand);
    let before = game.snapshot();
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    assert!(game.state.stack.is_empty());
    assert!(mana_actions(&game, land).is_empty());
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::PlayLand { card } if *card == second)));
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceCast { card } if *card == lions)));
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.snapshot(), after);
    game.begin_turn();
    assert!(!game.object_view(land).tapped);
    let actions = mana_actions(&game, land);
    assert_eq!(actions.len(), 2);
    let before = game.snapshot();
    let mut forged = actions[0].clone();
    if let Action::ActivateMana { production, .. } = &mut forged {
        *production = [0, 0, 0, 1, 0, 0];
    }
    assert_eq!(game.apply(Player::P0, forged), Err(Error::IllegalAction));
    assert_eq!(game.snapshot(), before);
    act(&mut game, actions[0].clone());
    assert!(game.object_view(land).tapped);
    assert_eq!(game.state.players[0].mana.iter().sum::<u32>(), 1);
    assert!(game.state.stack.is_empty());
}

#[test]
fn fast_and_slow_entry_thresholds_use_only_existing_own_other_lands() {
    for count in 0..=4 {
        for (name, tapped) in [("Blooming Marsh", count > 2), ("Deathcap Glade", count < 2)] {
            let mut game = scenario();
            for _ in 0..count {
                enter(&mut game, Player::P0, "Forest");
            }
            for _ in 0..4 {
                enter(&mut game, Player::P1, "Forest");
            }
            let id = enter(&mut game, Player::P0, name);
            assert_eq!(game.object_view(id).tapped, tapped, "{name}, {count}");
            // A zone change evaluates the condition again, with a fresh incarnation.
            let old = game.object_ref(id);
            game.move_object(id, Zone::Hand);
            enter(&mut game, Player::P0, "Forest");
            game.move_object(id, Zone::Battlefield);
            assert_ne!(game.object_ref(id), old);
            assert_eq!(
                game.object_view(id).tapped,
                if name == "Blooming Marsh" {
                    count + 1 > 2
                } else {
                    count + 1 < 2
                }
            );
        }
    }
}

#[test]
fn basic_supertype_includes_wastes_and_opponents_basics_do_not_count() {
    for existing in [
        None,
        Some("Azorius Guildgate"),
        Some("Wastes"),
        Some("Forest"),
    ] {
        let mut game = scenario();
        enter(&mut game, Player::P1, "Forest");
        if let Some(name) = existing {
            enter(&mut game, Player::P0, name);
        }
        let land = enter(&mut game, Player::P0, "Agna Qel'a");
        assert_eq!(
            game.object_view(land).tapped,
            !matches!(existing, Some("Wastes" | "Forest"))
        );
    }
    let wastes = Card::from_name("Wastes").unwrap();
    assert!(wastes.is_basic_land());
    assert_eq!(wastes.info().deck_limit, None);
    let game = Game::new(Config {
        decks: [vec![wastes; 60], vec![wastes; 60]],
        deck_rules: DeckRules::Standard,
        seed: 1,
        starting_player: Player::P0,
    })
    .unwrap();
    assert!(game.outcome().is_none());
}

#[test]
fn low_life_condition_checks_either_player_at_entry_not_later() {
    for player in [Player::P0, Player::P1] {
        for life in [14, 13, 1] {
            let mut game = scenario();
            game.state.players[player.index()].life = life;
            let id = enter(&mut game, Player::P0, "Abandoned Campground");
            assert_eq!(game.object_view(id).tapped, life > 13);
            game.state.players[player.index()].life = 20;
            assert_eq!(game.object_view(id).tapped, life > 13);
        }
    }
}

#[test]
fn simultaneous_entry_replacements_use_pre_event_battlefield() {
    for (name, existing, tapped) in [
        ("Blooming Marsh", 2, false),
        ("Deathcap Glade", 1, true),
        ("Agna Qel'a", 0, true),
    ] {
        let mut game = scenario();
        for _ in 0..existing {
            enter(&mut game, Player::P0, "Forest");
        }
        let basic = named(&mut game, Player::P0, "Forest", Zone::Graveyard);
        let a = named(&mut game, Player::P0, name, Zone::Graveyard);
        let b = named(&mut game, Player::P0, name, Zone::Graveyard);
        let before = game.snapshot();
        let events = game.move_objects_raw(&[basic, a, b], Zone::Battlefield);
        game.finish_zone_events(&events, &[]);
        assert_eq!(game.object_view(a).tapped, tapped);
        assert_eq!(game.object_view(b).tapped, tapped);
        let after = game.snapshot();
        game.restore(&before).unwrap();
        let events = game.move_objects_raw(&[basic, a, b], Zone::Battlefield);
        game.finish_zone_events(&events, &[]);
        assert_eq!(game.snapshot(), after);
    }
}

#[test]
fn land_entry_life_and_damage_are_stack_triggers_with_legal_targets() {
    let mut game = scenario();
    let cove = named(&mut game, Player::P0, "Tranquil Cove", Zone::Hand);
    act(&mut game, Action::PlayLand { card: cove });
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 21);
    let desert = enter(&mut game, Player::P0, "Abraded Bluffs");
    game.prepare_triggers();
    assert_eq!(game.actor(), Some(Player::P0));
    let actions = game.legal_actions();
    assert!(actions.iter().any(|a| matches!(
        a,
        Action::ChooseTarget {
            target: Target::Player(Player::P1)
        }
    )));
    assert!(!actions.iter().any(|a| matches!(
        a,
        Action::ChooseTarget {
            target: Target::Player(Player::P0)
        }
    )));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(game.state.stack[0].object, game.object_ref(desert));
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
}

#[test]
fn intrinsic_typed_land_mana_and_surveil_choices_are_private_and_replayable() {
    let mut game = scenario();
    let land = named(&mut game, Player::P0, "Commercial District", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    pass_pair(&mut game);
    assert!(game.observe(Player::P0).library_choice.is_some());
    assert!(game.observe(Player::P1).library_choice.is_none());
    let top = game
        .state
        .resolving
        .as_ref()
        .unwrap()
        .library
        .as_ref()
        .unwrap()
        .remaining[0];
    let before = game.snapshot();
    let action = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Graveyard,
    };
    act(&mut game, action.clone());
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), after);
    game.begin_turn();
    let productions: Vec<_> = mana_actions(&game, land)
        .into_iter()
        .map(|a| {
            if let Action::ActivateMana { production, .. } = a {
                production
            } else {
                unreachable!()
            }
        })
        .collect();
    assert_eq!(productions.len(), 2);
    assert!(productions.contains(&[0, 0, 0, 1, 0, 0]));
    assert!(productions.contains(&[0, 0, 0, 0, 1, 0]));
}

#[test]
fn land_sacrifice_cost_and_color_filtering_reuse_exact_funding() {
    let mut game = scenario();
    let cave = enter(&mut game, Player::P0, "Captivating Cave");
    let forest = enter(&mut game, Player::P0, "Forest");
    let lions = named(&mut game, Player::P0, "Savannah Lions", Zone::Hand);
    assert!(game.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceCast { card } if *card == lions)));
    act(&mut game, Action::TapMana { card: forest });
    let filter = mana_actions(&game, cave)
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ActivateMana {
                    ability: 1,
                    production: [1, 0, 0, 0, 0, 0],
                    ..
                }
            )
        })
        .unwrap();
    act(&mut game, filter);
    assert_eq!(game.state.players[0].mana, [1, 0, 0, 0, 0, 0]);
    assert!(game.object_view(cave).tapped);
    let drawland = enter(&mut game, Player::P0, "Airship Engine Room");
    game.begin_turn();
    game.state.players[0].mana = [4, 0, 0, 0, 0, 0];
    let draw = game
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::Activate { card, ability: 1, .. } if *card == drawland)
        })
        .unwrap();
    let old = game.object_ref(drawland);
    let count = game.hand_count(Player::P0);
    act(&mut game, draw);
    assert_eq!(game.object_view(drawland).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.last().unwrap().object, old);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), count + 1);
}

#[test]
fn paired_exile_returns_apply_all_entry_conditions_before_any_return() {
    let mut game = scenario();
    enter(&mut game, Player::P0, "Forest");
    let sources: Vec<_> = (0..3)
        .map(|_| named(&mut game, Player::P1, "Banishing Light", Zone::Battlefield))
        .collect();
    let names = ["Forest", "Deathcap Glade", "Deathcap Glade"];
    let held: Vec<_> = names
        .into_iter()
        .map(|name| named(&mut game, Player::P0, name, Zone::Exile))
        .collect();
    // Private kernel fixture: wider land-exiling cards remain unsupported, but
    // every completed simultaneous-return batch must obey entry replacements.
    for (&source, &exiled) in sources.iter().zip(&held) {
        game.state.exile_links.push(ExileLink {
            source: game.object_ref(source),
            exiled: game.object_ref(exiled),
            return_zone: Zone::Battlefield,
        });
    }
    let before = game.snapshot();
    let events = game.move_objects_raw(&sources, Zone::Graveyard);
    game.finish_zone_events(&events, &[]);
    for &land in &held[1..] {
        assert!(game.object_view(land).tapped);
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    }
    assert!(game.state.exile_links.is_empty());
    let after = game.snapshot();
    game.restore(&before).unwrap();
    let events = game.move_objects_raw(&sources, Zone::Graveyard);
    game.finish_zone_events(&events, &[]);
    assert_eq!(game.snapshot(), after);
}
