use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}
fn factory(name: &str) -> Card {
    Card::catalogue()
        .find(|card| card.token_info().is_some_and(|info| info.name == name))
        .unwrap()
}
fn mana_action(game: &Game, id: CardId, production: Mana) -> Action {
    game.legal_actions().into_iter().find(|action|
        matches!(action, Action::ActivateMana { card, production: actual, .. } if *card == id && *actual == production)).unwrap()
}
fn token(game: &mut Game, name: &str) -> CardId {
    let card = factory(name);
    game.create_tokens(Player::P0, card, 1, false);
    game.state
        .objects
        .iter()
        .position(|object| object.zone == Zone::Battlefield && object.card == card)
        .unwrap() as CardId
}

#[test]
fn colored_and_colorless_costs_are_distinct_and_generic_can_use_either() {
    let mut choices = Vec::new();
    payments([0, 0, 0, 0, 1, 0], [0, 0, 0, 0, 0, 1], 0, |p| {
        choices.push(p)
    });
    assert!(choices.is_empty());
    payments([0, 0, 0, 0, 0, 1], [0, 0, 0, 0, 1, 0], 0, |p| {
        choices.push(p)
    });
    assert!(choices.is_empty());
    payments([0, 0, 0, 0, 1, 1], [0; 6], 1, |p| choices.push(p));
    choices.sort();
    assert_eq!(choices, vec![[0, 0, 0, 0, 0, 1], [0, 0, 0, 0, 1, 0]]);
}

#[test]
fn mana_abilities_resolve_without_stack_and_retain_priority_with_an_existing_spell() {
    let mut game = scenario();
    let lotus = named(&mut game, "Gilded Lotus", Zone::Battlefield);
    let bolt = spawn(&mut game, Player::P0, Card::LightningBolt, Zone::Hand);
    game.state.players[0].mana[3] = 1;
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    let stack = game.state.stack.clone();
    let before = game.snapshot();
    let action = mana_action(&game, lotus, [0, 0, 0, 0, 3, 0]);
    act(&mut game, action.clone());
    assert_eq!(game.state.stack, stack);
    assert_eq!(game.actor(), Some(Player::P0));
    assert_eq!(game.state.players[0].mana[4], 3);
    assert!(game.object_view(lotus).tapped);
    assert_eq!(game.state.players[1].life, 20);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), after);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 17);
}

#[test]
fn creature_mana_obeys_sickness_and_any_color_excludes_colorless() {
    let mut game = scenario();
    let druid = named(&mut game, "Great Forest Druid", Zone::Hand);
    game.move_object(druid, Zone::Battlefield);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ActivateMana { .. }))
    );
    game.begin_turn();
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::ActivateMana { .. }))
        .collect();
    assert_eq!(choices.len(), 5);
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::ActivateMana {
                card: druid,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    act(&mut game, choices[0].clone());
    assert!(game.object_view(druid).tapped);
}

#[test]
fn colorless_mana_pays_generic_activation_cost_and_once_per_turn_filter_is_enforced() {
    let mut game = scenario();
    let archive = named(&mut game, "Hedron Archive", Zone::Battlefield);
    let scarecrow = named(&mut game, "Scarecrow Guide", Zone::Battlefield);
    let action = mana_action(&game, archive, [0, 0, 0, 0, 0, 2]);
    act(&mut game, action);
    let action = mana_action(&game, scarecrow, [0, 1, 0, 0, 0, 0]);
    act(&mut game, action);
    assert_eq!(game.state.players[0].mana, [0, 1, 0, 0, 0, 1]);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == scarecrow))
    );
    game.begin_turn();
    assert!(
        game.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == scarecrow))
    );
}

#[test]
fn treasure_is_noncreature_sacrifices_for_immediate_mana_and_reclaims_its_slot() {
    let mut game = scenario();
    let treasure = token(&mut game, "Treasure Token");
    assert!(!game.state.objects[treasure as usize].card.is_creature());
    assert!(game.object_view(treasure).summoning_sick); // Does not restrict artifact tap costs.
    let before = game.snapshot();
    let action = mana_action(&game, treasure, [0, 0, 0, 1, 0, 0]);
    act(&mut game, action.clone());
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[0].mana[3], 1);
    assert_eq!(game.state.objects[treasure as usize].zone, Zone::Outside);
    assert_eq!(game.state.free_tokens, vec![treasure]);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn clue_ignores_tapped_status_food_requires_untapped_and_both_use_stack() {
    let mut game = scenario();
    let clue = token(&mut game, "Clue Token");
    game.state.objects[clue as usize].tapped = true;
    game.state.players[0].mana[5] = 2;
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Activate { card, .. } if *card == clue))
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.hand_count(Player::P0), 0);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.objects[clue as usize].zone, Zone::Outside);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    let food = token(&mut game, "Food Token");
    game.state.players[0].mana[5] = 2;
    game.state.objects[food as usize].tapped = true;
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Activate { card, .. } if *card == food))
    );
    game.state.objects[food as usize].tapped = false;
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Activate { card, .. } if *card == food))
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.state.players[0].life, 20);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 23);
}

#[test]
fn mutagen_is_sorcery_only_targeted_and_does_not_require_creature_sickness_check() {
    let mut game = scenario();
    let mutagen = token(&mut game, "Mutagen Token");
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    game.state.players[0].mana[5] = 1;
    game.state.step = Step::Upkeep;
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Activate { .. }))
    );
    game.state.step = Step::Main1;
    let action = game.legal_actions().into_iter().find(|a|
        matches!(a, Action::Activate { card, target:Some(Target::Card(id)), .. } if *card == mutagen && *id == bears)).unwrap();
    act(&mut game, action);
    assert_eq!(game.object_view(bears).counters.plus_one, 0);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).counters.plus_one, 1);
}

#[test]
fn resolution_mana_choice_is_private_owned_and_replayable_before_later_instructions() {
    let mut game = scenario();
    let source = spawn(&mut game, Player::P0, Card::Divination, Zone::Hand);
    let item = game.cast_item(Player::P0, source, [0; 6]);
    // Direct kernel fixture for a choice followed by more resolution work.
    game.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: &[
            Instruction::AddMana(ManaProduction::Choice { mask: 3, amount: 1 }),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        search: None,
        choice: None,
    });
    game.run_program();
    assert_eq!(game.actor(), Some(Player::P0));
    assert_eq!(game.observe(Player::P0).mana_choices.len(), 2);
    assert!(game.observe(Player::P1).mana_choices.is_empty());
    assert_eq!(game.hand_count(Player::P0), 0);
    let before = game.snapshot();
    let forged = Action::ChooseMana {
        production: [2, 0, 0, 0, 0, 0],
    };
    assert_eq!(game.apply(Player::P0, forged), Err(Error::IllegalAction));
    assert_eq!(game.snapshot(), before);
    let chosen = Action::ChooseMana {
        production: [0, 1, 0, 0, 0, 0],
    };
    act(&mut game, chosen.clone());
    assert_eq!(game.state.players[0].mana[1], 1);
    assert_eq!(game.hand_count(Player::P0), 1);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, chosen);
    assert_eq!(game.snapshot(), after);
}
