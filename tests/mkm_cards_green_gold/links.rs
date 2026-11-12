use super::*;

const CLUE: u16 = 5296;
const IMP: u16 = 5684;
const SPIRIT: u16 = 5477;

#[test]
fn alquist_proft_pays_x_and_sacrifices_a_clue_for_equal_draw_and_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Alquist Proft, Master Sleuth",
        Zone::Hand,
    );
    let artifact = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
    assert!(game.keywords(source).contains(Keywords::VIGILANCE));
    let clue = tokens(&game, player, CLUE)[0];
    reject(&mut game, activation(source, 0));
    game.state.objects[source as usize].summoning_sick = false;
    for x in [0, 2] {
        let mut branch = game.clone();
        act(&mut branch, activation(source, 0));
        for _ in 0..6 {
            match branch.state.pending {
                Pending::X { .. } => act(&mut branch, Action::ChooseX { value: x }),
                Pending::AdditionalCost { .. } => {
                    let object = branch.object_ref(artifact);
                    reject(&mut branch, Action::ChooseAdditionalCost { object });
                    let object = branch.object_ref(clue);
                    act(&mut branch, Action::ChooseAdditionalCost { object });
                }
                Pending::PayCosts { .. } => pay(&mut branch),
                Pending::Priority => break,
                ref pending => panic!("unexpected Alquist payment state {pending:?}"),
            }
        }
        assert!(branch.object_view(source).tapped);
        assert!(tokens(&branch, player, CLUE).is_empty());
        settle(&mut branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            u64::from(x)
        );
        assert_eq!(
            branch.state.players[player.index()].life,
            20 + i128::from(x)
        );
    }
}

#[test]
fn coerced_to_kill_steals_the_creature_and_layers_assassin_deathtouch_and_base_stats() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Coerced to Kill", Zone::Hand);
    let target = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    game.add_counter(target, CounterKind::PlusOne, 1);
    cast(&mut game, aura, Some(Target::Card(target)));
    settle(&mut game);
    let creature = game.object_view(target);
    assert_eq!(creature.controller, player);
    assert!(creature.has_subtype("Giant"));
    assert!(creature.has_subtype("Assassin"));
    assert!(creature.keywords.contains(Keywords::DEATHTOUCH));
    assert_eq!(stats(&game, target), (2, 2));
    game.move_object(aura, Zone::Graveyard);
    let creature = game.object_view(target);
    assert_eq!(creature.controller, opponent);
    assert!(creature.has_subtype("Giant"));
    assert!(!creature.has_subtype("Assassin"));
    assert!(!creature.keywords.contains(Keywords::DEATHTOUCH));
    assert_eq!(stats(&game, target), (4, 4));
}

#[test]
fn judith_modifies_the_triggering_spell_or_creates_an_imp_with_a_real_death_trigger() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let judith = put(&mut game, player, "Judith, Carnage Connoisseur", Zone::Hand);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, judith, None);
    settle(&mut game);
    assert_eq!(stats(&game, judith), (3, 4));
    cast(&mut game, shock, Some(Target::Card(victim)));

    let mut imp = game.clone();
    act(&mut imp, Action::ChooseMode { mode: 1 });
    settle(&mut imp);
    assert_eq!(imp.object_view(victim).damage, 2);
    assert_eq!(imp.state.players[player.index()].life, 20);
    let token = tokens(&imp, player, IMP)[0];
    assert_eq!(stats(&imp, token), (2, 2));
    assert_eq!(imp.object_view(token).colors, 8);
    let mut exiled = imp.clone();
    exiled.move_object(token, Zone::Exile);
    exiled.prepare_triggers();
    assert!(exiled.state.stack.is_empty());
    imp.sacrifice_batch(&[(player, token)]);
    imp.prepare_triggers();
    settle(&mut imp);
    assert_eq!(imp.state.players[opponent.index()].life, 18);

    act(&mut game, Action::ChooseMode { mode: 0 });
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert!(tokens(&game, player, IMP).is_empty());
}

#[test]
fn kaya_resolves_all_three_loyalty_choices_and_copies_one_eligible_exiled_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let kaya = put(&mut game, player, "Kaya, Spirits' Justice", Zone::Hand);
    let ally = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let card = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let keep = put(&mut game, player, "Island", Zone::Library);
    let mill = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, kaya, None);
    settle(&mut game);
    assert_eq!(game.counter_count(kaya, CounterKind::Loyalty), 3);

    let mut spirit = game.clone();
    act(&mut spirit, activation(kaya, 1));
    pay(&mut spirit);
    settle(&mut spirit);
    assert_eq!(spirit.counter_count(kaya, CounterKind::Loyalty), 4);
    let new = tokens(&spirit, player, SPIRIT)[0];
    assert_eq!(stats(&spirit, new), (1, 1));
    assert_eq!(spirit.object_view(new).colors, 5);
    assert!(spirit.keywords(new).contains(Keywords::FLYING));

    let mut surveil = game.clone();
    act(&mut surveil, activation(kaya, 0));
    pay(&mut surveil);
    pass_pair(&mut surveil);
    act(
        &mut surveil,
        Action::ArrangeLibrary {
            card: mill,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut surveil,
        Action::ArrangeLibrary {
            card: keep,
            destination: LibraryDestination::Top,
        },
    );
    let object = surveil.object_ref(card);
    act(&mut surveil, Action::SelectResolutionExile { object });
    settle(&mut surveil);
    assert_eq!(surveil.counter_count(kaya, CounterKind::Loyalty), 5);
    assert_eq!(surveil.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(surveil.object_view(card).zone, Zone::Exile);

    game.create_tokens(player, Card::from_catalogue_id(CLUE), 1, false);
    assert!(!game.advance_entries());
    let token = tokens(&game, player, CLUE)[0];
    act(&mut game, activation(kaya, 2));
    act(&mut game, target(ally));
    act(&mut game, target(enemy));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(kaya, CounterKind::Loyalty), 1);
    for card in [ally, enemy] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    act(&mut game, target(token));
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(tokens(&declined, player, CLUE), [token]);
    let object = game.object_ref(enemy);
    reject(&mut game, Action::SelectResolutionExile { object });
    let object = game.object_ref(ally);
    act(&mut game, Action::SelectResolutionExile { object });
    settle(&mut game);
    assert_eq!(stats(&game, token), (3, 3));
    assert!(game.keywords(token).contains(Keywords::FLYING));
    assert!(game.object_view(token).has_subtype("Giant"));
    game.finish_cleanup();
    assert_eq!(tokens(&game, player, CLUE), [token]);
}

#[test]
fn tail_the_suspect_adds_a_land_play_then_kellan_attacks_and_draws_for_its_own_artifact() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Kellan, Inquisitive Prodigy // Tail the Suspect",
        Zone::Hand,
    );
    let forest = put(&mut game, player, "Forest", Zone::Hand);
    let island = put(&mut game, player, "Island", Zone::Hand);
    let enemy_artifact = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: forest });
    reject(&mut game, Action::PlayLand { card: island });
    act(&mut game, Action::AnnounceAdventure { card: source });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    let clue = tokens(&game, player, CLUE)[0];
    act(&mut game, Action::PlayLand { card: island });
    act(&mut game, Action::AnnounceExile { card: source });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 4));
    assert!(
        game.keywords(source)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.state.objects[source as usize].summoning_sick = false;
    game.state.objects[clue as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    attack(&mut game, &[source]);
    assert!(!game.object_view(source).tapped);

    let mut enemy = game.clone();
    act(&mut enemy, target(enemy_artifact));
    if enemy.legal_actions().contains(&Action::FinishTargets) {
        act(&mut enemy, Action::FinishTargets);
    }
    settle(&mut enemy);
    assert_eq!(enemy.object_view(enemy_artifact).zone, Zone::Graveyard);
    assert_eq!(enemy.state.players[player.index()].cards_drawn_this_turn, 0);

    act(&mut game, target(clue));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE), [clue]);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

fn all_basic_types(game: &Game, land: CardId) -> bool {
    let view = game.object_view(land);
    ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .into_iter()
        .all(|subtype| view.has_subtype(subtype))
}

#[test]
fn leyline_of_the_guildpact_accepts_every_printed_hybrid_assignment() {
    let player = Player::P0;
    for mask in 0u8..16 {
        let mut game = scenario(player);
        let leyline = put(&mut game, player, "Leyline of the Guildpact", Zone::Hand);
        let mut payment = [0; 6];
        for (color, mana) in payment.iter_mut().take(4).enumerate() {
            *mana = u32::from((mask >> color) & 1);
        }
        payment[4] = 4 - mask.count_ones();
        game.state.players[player.index()].mana = payment;
        act(&mut game, Action::AnnounceCast { card: leyline });
        if let Some(option) = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::ChooseSpellCost { .. }))
        {
            act(&mut game, option);
        }
        if game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::ChooseCastCost { .. }))
        {
            choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            });
        }
        act(&mut game, Action::PayCosts { payment });
        settle(&mut game);
        assert_eq!(game.object_view(leyline).zone, Zone::Battlefield);
    }
    let mut game = scenario(player);
    let leyline = put(&mut game, player, "Leyline of the Guildpact", Zone::Hand);
    game.state.players[player.index()].mana = [2, 0, 0, 0, 2, 0];
    reject(&mut game, Action::AnnounceCast { card: leyline });
}

#[test]
fn leyline_of_the_guildpact_can_start_in_play_and_gives_live_types_colors_and_mana() {
    let player = Player::P1;
    let leyline = Card::from_name("Leyline of the Guildpact").unwrap();
    let mut deck = vec![Card::Forest; 6];
    deck.push(leyline);
    let mut opening = Game::new(Config {
        decks: [vec![Card::Forest; 7], deck],
        seed: 20261023,
        starting_player: player,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    act(&mut opening, Action::Keep);
    act(&mut opening, Action::Keep);
    let card = opening
        .legal_actions()
        .into_iter()
        .find_map(|action| match action {
            Action::BeginOnBattlefield { object } => Some(object),
            _ => None,
        })
        .unwrap();
    act(&mut opening, Action::BeginOnBattlefield { object: card });
    act(&mut opening, Action::FinishOpeningHand);
    act(&mut opening, Action::FinishOpeningHand);
    assert_eq!(opening.object_view(card.card).zone, Zone::Battlefield);
    assert_eq!(opening.object_view(card.card).colors, 31);

    let mut game = scenario(player);
    let source = spawn(&mut game, player, leyline, Zone::Battlefield);
    let cave = put(&mut game, player, "Hidden Courtyard", Zone::Battlefield);
    let artifact = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    assert!(all_basic_types(&game, cave));
    assert!(game.object_view(cave).has_subtype("Cave"));
    assert_eq!(game.object_view(cave).colors, 0);
    assert_eq!(game.object_view(artifact).colors, 31);
    assert!(!all_basic_types(&game, enemy));
    let blue = game.state.players[player.index()].mana[1];
    let grantor = game.object_ref(cave);
    act(
        &mut game,
        Action::TapGrantedMana {
            card: cave,
            grantor,
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].mana[1], blue + 1);
    game.record_control(game.object_ref(source), player.other(), None);
    assert!(!all_basic_types(&game, cave));
    assert_eq!(game.object_view(artifact).colors, 0);
    assert!(all_basic_types(&game, enemy));
}
