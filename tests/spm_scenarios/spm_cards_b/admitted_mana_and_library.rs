use super::*;
use rustmage_core::cards::authored::CardRules;

fn produce(game: &mut Game, card: CardId, mana: Mana) {
    choose(game, |action| {
        matches!(action, Action::ActivateMana { card: source, production, .. }
            if *source == card && *production == mana)
    });
}

fn next_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    advance_until(game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::Main1
    });
}

#[test]
fn web_watch_exiles_two_and_its_mixed_mana_pays_only_for_spells_cast_from_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let watch = put(&mut game, player, "Interdimensional Web Watch", Zone::Hand);
    cast(&mut game, watch, None);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);

    let exile_spell = put(&mut game, player, "Shock", Zone::Exile);
    let hand_spell = put(&mut game, player, "Shock", Zone::Hand);
    game.grant_exile_play_permission(player, game.object_ref(exile_spell), false);
    game.state.players[player.index()].mana = [0; 6];
    let produced = [0, 1, 0, 1, 0, 0];
    produce(&mut game, watch, produced);
    assert_eq!(game.state.players[player.index()].mana, produced);
    assert!(game.object_view(watch).tapped);
    reject(&mut game, Action::AnnounceCast { card: hand_spell });
    act(&mut game, Action::AnnounceExile { card: exile_spell });
    choose_target(&mut game, Target::Player(player.other()));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);

    next_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    let mut use_permission = game.clone();
    act(&mut use_permission, Action::PlayLand { card: land });
    act(
        &mut use_permission,
        Action::AnnounceExile { card: creature },
    );
    pay(&mut use_permission);
    settle(&mut use_permission);
    assert_eq!(use_permission.object_view(land).zone, Zone::Battlefield);
    assert_eq!(use_permission.object_view(creature).zone, Zone::Battlefield);

    next_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    reject(&mut game, Action::PlayLand { card: land });
    reject(&mut game, Action::AnnounceExile { card: creature });
}

#[test]
fn island_mountain_and_plains_have_unlimited_standard_copies_and_intrinsic_mana() {
    let player = Player::P1;
    for (name, color) in [("Island", 1), ("Mountain", 3), ("Plains", 0)] {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_basic_land());
        assert!(
            Game::new(Config {
                decks: [vec![card; 60], vec![Card::Forest; 60]],
                seed: 52,
                starting_player: player,
                deck_rules: DeckRules::Standard,
            })
            .is_ok(),
            "{name}"
        );
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let id = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: id });
        act(&mut game, Action::TapMana { card: id });
        let mut expected = [0; 6];
        expected[color] = 1;
        assert_eq!(game.state.players[player.index()].mana, expected);
        assert!(game.object_view(id).tapped);
    }
}

#[test]
fn morbius_exiles_itself_to_take_one_private_card_and_order_the_bottom() {
    let player = Player::P0;
    let mut game = scenario(player);
    let morbius = put(&mut game, player, "Morbius the Living Vampire", Zone::Hand);
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let selected = put(&mut game, player, "Hill Giant", Zone::Library);
    cast(&mut game, morbius, None);
    settle(&mut game);
    assert!(
        game.object_view(morbius).keywords.contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::LIFELINK)
        )
    );
    cast(&mut game, spell, Some(Target::Card(morbius)));
    settle(&mut game);

    game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 0];
    announce_ability(&mut game, morbius, 0);
    pay(&mut game);
    assert_eq!(game.object_view(morbius).zone, Zone::Exile);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: selected });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    act(&mut game, Action::OrderLibraryBottom { card: land });
    act(&mut game, Action::OrderLibraryBottom { card: shock });
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library[..2],
        [shock, land]
    );
    assert_eq!(game.object_view(selected).zone, Zone::Hand);
    assert!(
        !game
            .observe(player.other())
            .known_hand
            .contains(&game.object_ref(selected))
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn multiversal_passage_chooses_its_land_type_before_payment_for_untapped_entry() {
    const REMOVE_ABILITIES: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            &[Instruction::RemoveAbilities {
                subject: Subject::Targets,
                duration: AnimationDuration::Permanent,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &REMOVE_ABILITIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Multiversal Passage", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    reject(&mut game, Action::ChooseEntryLife { pay: true });
    let mut tapped = game.clone();
    act(
        &mut tapped,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::BasicLandType(BasicLandType::Mountain),
        },
    );
    act(&mut tapped, Action::ChooseEntryLife { pay: false });
    assert!(tapped.object_view(land).tapped);
    assert!(tapped.has_subtype(land, "Mountain"));
    assert_eq!(tapped.state.players[player.index()].life, 20);

    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::BasicLandType(BasicLandType::Island),
        },
    );
    act(&mut game, Action::ChooseEntryLife { pay: true });
    assert!(!game.object_view(land).card.is_basic_land());
    assert!(game.has_subtype(land, "Island"));
    assert_eq!(game.state.players[player.index()].life, 18);
    game.state.players[player.index()].mana = [0; 6];
    let blue = [0, 1, 0, 0, 0, 0];
    let mana = Action::TapGrantedMana {
        card: land,
        grantor: game.object_ref(land),
        production: blue,
    };
    let spell = put(&mut game, player, "Spectral Sailor", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };

    let mut direct = game.clone();
    act(&mut direct, mana.clone());
    assert_eq!(direct.state.players[player.index()].mana, blue);
    reject(&mut direct, mana.clone());

    let mut blanked = game.clone();
    blanked.state.players[player.index()].mana = blue;
    let removal = put(&mut blanked, player, "Unsummon", Zone::Hand);
    cast(&mut blanked, removal, Some(Target::Card(land)));
    settle(&mut blanked);
    assert!(blanked.has_subtype(land, "Island"));
    assert!(!blanked.object_view(land).tapped);
    reject(&mut blanked, mana.clone());
    reject(&mut blanked, announce.clone());

    act(&mut game, announce);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| action == &mana)
        .expect("chosen Island can fund the spell");
    let mut raw = game.clone();
    act(&mut raw, mana);
    game.apply_index(player, game.revision(), index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn ominous_asylum_enters_tapped_and_offers_black_red_or_paid_surveil() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Ominous Asylum", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    next_main(&mut game, player);
    assert!(!game.object_view(land).tapped);
    for color in [2, 3] {
        let mut mana = [0; 6];
        mana[color] = 1;
        let mut branch = game.clone();
        produce(&mut branch, land, mana);
        assert_eq!(branch.state.players[player.index()].mana, mana);
    }

    game.state.players[player.index()].mana[5] = 4;
    let top = *game.state.players[player.index()].library.last().unwrap();
    announce_ability(&mut game, land, 1);
    pay(&mut game);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_choice.is_none());
    for destination in [LibraryDestination::Graveyard, LibraryDestination::Top] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: top,
                destination,
            },
        );
        settle(&mut branch);
        assert_eq!(
            branch.object_view(top).zone,
            if destination == LibraryDestination::Top {
                Zone::Library
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn radioactive_spider_pays_and_sacrifices_to_find_only_a_spider_hero() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Radioactive Spider", Zone::Battlefield);
    let spider_only = put(&mut game, player, "Radioactive Spider", Zone::Library);
    let hero_only = put(
        &mut game,
        player,
        "Peter Parker // Amazing Spider-Man",
        Zone::Library,
    );
    let both = put(
        &mut game,
        player,
        "Iron Spider, Stark Upgrade",
        Zone::Library,
    );
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::REACH.union(Keywords::DEATHTOUCH))
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    announce_ability(&mut game, source, 0);
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    for card in [spider_only, hero_only] {
        reject(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::ChooseSearch { card: both });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(both).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(both))
    );
}
