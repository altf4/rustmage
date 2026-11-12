use super::support::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_ability(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn activate(game: &mut Game, source: CardId, index: u16) {
    choose(game, |action| {
        matches!(
            action,
            Action::Activate { card, ability, .. } | Action::AnnounceActivate { card, ability, .. }
            if *card == source && *ability == index
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn mana(game: &mut Game, card: CardId, ability: u16, production: Mana) {
    act(
        game,
        Action::ActivateMana {
            card,
            ability,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn arrange(game: &mut Game, card: CardId, destination: LibraryDestination) {
    act(game, Action::ArrangeLibrary { card, destination });
}

#[test]
fn cryptex_earns_the_fifth_unlock_with_evidence_then_surveils_and_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cryptex = put(&mut game, player, "Cryptex", Zone::Battlefield);
    game.set_counters(cryptex, Counters::of(CounterKind::Unlock, 4));
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: cryptex,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let evidence = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    act(
        &mut game,
        Action::AnnounceMana {
            card: cryptex,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0; 6],
        },
    );
    let object = game.object_ref(evidence);
    act(&mut game, Action::ChooseEvidence { object });
    act(&mut game, Action::FinishEvidence);
    pay(&mut game);
    act(&mut game, Action::ChooseMana { production: BLUE });
    assert_eq!(game.object_view(evidence).zone, Zone::Exile);
    assert!(game.object_view(cryptex).tapped);
    assert_eq!(game.counter_count(cryptex, CounterKind::Unlock), 5);

    let kept = put(&mut game, player, "Mountain", Zone::Library);
    let kept_second = put(&mut game, player, "Forest", Zone::Library);
    let milled = put(&mut game, player, "Shock", Zone::Library);
    activate(&mut game, cryptex, 1);
    assert_eq!(game.object_view(cryptex).zone, Zone::Graveyard);
    pass_pair(&mut game);
    arrange(&mut game, milled, LibraryDestination::Graveyard);
    arrange(&mut game, kept, LibraryDestination::Top);
    arrange(&mut game, kept_second, LibraryDestination::Top);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    for id in [kept, kept_second] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn lumbering_laundry_turns_up_and_its_paid_look_applies_through_cleanup() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let laundry = put(&mut game, player, "Lumbering Laundry", Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: laundry });
    pay(&mut game);
    settle(&mut game);
    let object = game.object_ref(laundry);
    act(
        &mut game,
        Action::BeginFaceUp {
            object,
            cost: ManaCost {
                colored: [0; 6],
                generic: 5,
            },
        },
    );
    choose(&mut game, |a| matches!(a, Action::PayFaceUp { .. }));
    assert_eq!(
        (game.stats(laundry).power, game.stats(laundry).toughness),
        (4, 5)
    );

    let first = put(&mut game, opponent, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    let known = |game: &Game, id| {
        game.observe(player)
            .face_down_permanents
            .into_iter()
            .find(|row| row.object.card == id)
            .unwrap()
            .underlying
    };
    assert_eq!(known(&game, first), None);
    activate(&mut game, laundry, 0);
    settle(&mut game);
    assert_eq!(known(&game, first), Card::from_name("Grizzly Bears"));
    let during = put(&mut game, opponent, "Bear Cub", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    assert_eq!(known(&game, during), Card::from_name("Bear Cub"));

    game.state.step = Step::End;
    pass_pair(&mut game);
    let later = put(&mut game, opponent, "Hill Giant", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    assert_eq!(known(&game, later), None);
    assert_eq!(known(&game, first), Card::from_name("Grizzly Bears"));
}

#[test]
fn magnetic_snuffler_brings_its_equipment_attached_and_sees_artifact_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bow = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let wrong = put(&mut game, player, "Candy Trail", Zone::Graveyard);
    let snuffler = put(&mut game, player, "Magnetic Snuffler", Zone::Hand);
    cast(&mut game, snuffler, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    target(&mut game, bow);
    settle(&mut game);
    assert_eq!(
        game.object_view(bow).attached_to,
        Some(game.object_ref(snuffler))
    );
    assert_eq!(game.stats(snuffler).power, 5);
    game.create_tokens(player, CLUE, 1, false);
    assert!(!game.advance_entries());
    let clue = game.controlled_token_ids(player).next().unwrap();
    activate(&mut game, clue, 0);
    settle(&mut game);
    assert_eq!(game.counter_count(snuffler, CounterKind::PlusOne), 1);
    assert_eq!(game.stats(snuffler).power, 6);
}

#[test]
fn thinking_cap_has_a_detective_only_discount_and_general_equip() {
    let player = Player::P1;
    let mut game = scenario(player);
    let detective = put(&mut game, player, "Novice Inspector", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let cap = put(&mut game, player, "Thinking Cap", Zone::Battlefield);
    game.state.players[player.index()].mana = COLORLESS;
    announce_ability(&mut game, cap, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    target(&mut game, detective);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        (game.stats(detective).power, game.stats(detective).toughness),
        (2, 4)
    );

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    announce_ability(&mut game, cap, 1);
    target(&mut game, bear);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(cap).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!((game.stats(bear).power, game.stats(bear).toughness), (3, 4));
    assert_eq!(game.stats(detective).power, 1);
}

#[test]
fn branch_can_disguise_as_a_spell_and_retains_exactly_its_turn_up_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let branch = put(&mut game, player, "Branch of Vitu-Ghazi", Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: branch });
    pay(&mut game);
    settle(&mut game);
    let object = game.object_ref(branch);
    assert!(game.is_face_down(object));
    assert!(game.object_view(branch).is_creature());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    act(
        &mut game,
        Action::BeginFaceUp {
            object,
            cost: ManaCost {
                colored: [0; 6],
                generic: 3,
            },
        },
    );
    choose(&mut game, |a| matches!(a, Action::PayFaceUp { .. }));
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 2, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.object_ref(branch), object);
    assert_eq!(game.object_view(branch).kinds, 2);
    mana(&mut game, branch, 0, COLORLESS);
    assert_eq!(game.state.players[player.index()].mana, [0, 2, 0, 0, 0, 1]);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0, 2, 0, 0, 0, 0]);
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn public_thoroughfare_can_tap_a_sick_artifact_or_be_sacrificed() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Public Thoroughfare", Zone::Hand);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.create_tokens(player, Card::from_catalogue_id(5266), 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(player).next().unwrap();
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    pass_pair(&mut game);
    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    assert_eq!(unpaid.object_view(land).zone, Zone::Graveyard);
    let chosen = |g: &Game, card| Action::ChooseAdditionalCost {
        object: g.object_ref(card),
    };
    let invalid = chosen(&game, bear);
    reject(&mut game, invalid);
    let valid = chosen(&game, token);
    act(&mut game, valid);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert!(game.on_battlefield(land));
    assert!(game.object_view(token).tapped);
    game.state.objects[land as usize].tapped = false;
    let before = game.state.players[player.index()].mana[1];
    mana(&mut game, land, 0, BLUE);
    assert_eq!(game.state.players[player.index()].mana[1], before + 1);
}

#[test]
fn scene_of_the_crime_is_a_clue_land_with_both_mana_abilities_and_a_draw() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let scene = put(&mut game, player, "Scene of the Crime", Zone::Hand);
    act(&mut game, Action::PlayLand { card: scene });
    assert!(game.object_view(scene).tapped);
    assert_eq!(game.object_view(scene).kinds, 2 | 16);
    assert!(game.object_view(scene).has_subtype("Clue"));
    game.state.objects[scene as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    mana(&mut game, scene, 0, COLORLESS);
    assert_eq!(game.state.players[player.index()].mana, COLORLESS);

    game.state.objects[scene as usize].tapped = false;
    let witness = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[witness as usize].summoning_sick = true;
    let wrong = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceMana {
            card: scene,
            ability: 1,
            counter_payment: Counters::default(),
            production: BLUE,
        },
    );
    let choose_object = |g: &Game, id| Action::ChooseAdditionalCost {
        object: g.object_ref(id),
    };
    let invalid = choose_object(&game, wrong);
    reject(&mut game, invalid);
    let valid = choose_object(&game, witness);
    act(&mut game, valid);
    pay(&mut game);
    assert!(game.object_view(scene).tapped);
    assert!(game.object_view(witness).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 1]);
    activate(&mut game, scene, 2);
    settle(&mut game);
    assert_eq!(game.object_view(scene).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn every_mkm_surveillance_land_has_both_printed_types_and_surveils_on_tapped_entry() {
    let player = Player::P0;
    for (name, first, second) in [
        ("Commercial District", (3, "Mountain"), (4, "Forest")),
        ("Elegant Parlor", (3, "Mountain"), (0, "Plains")),
        ("Hedge Maze", (4, "Forest"), (1, "Island")),
        ("Lush Portico", (4, "Forest"), (0, "Plains")),
        ("Meticulous Archive", (0, "Plains"), (1, "Island")),
        ("Raucous Theater", (2, "Swamp"), (3, "Mountain")),
        ("Shadowy Backstreet", (2, "Swamp"), (0, "Plains")),
        ("Thundering Falls", (1, "Island"), (3, "Mountain")),
        ("Undercity Sewers", (1, "Island"), (2, "Swamp")),
        ("Underground Mortuary", (2, "Swamp"), (4, "Forest")),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        let milled = put(&mut game, player, "Shock", Zone::Library);
        act(&mut game, Action::PlayLand { card: land });
        let view = game.object_view(land);
        assert!(view.tapped, "{name}");
        let colors = [first.0, second.0];
        for (index, subtype) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                view.has_subtype(subtype),
                colors.contains(&index),
                "{name} {subtype}"
            );
        }
        pass_pair(&mut game);
        arrange(&mut game, milled, LibraryDestination::Graveyard);
        assert_eq!(game.object_view(milled).zone, Zone::Graveyard, "{name}");
        for (index, _) in [first, second] {
            let mut branch = game.clone();
            branch.state.objects[land as usize].tapped = false;
            branch.state.players[player.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[index] = 1;
            choose(&mut branch, |action| {
                matches!(
                    action, Action::ActivateMana { card, production: offered, .. }
                        if *card == land && *offered == production
                )
            });
            assert_eq!(
                branch.state.players[player.index()].mana,
                production,
                "{name}"
            );
        }
    }
}

#[test]
fn mkm_basic_lands_enter_untapped_and_produce_their_own_colors() {
    let player = Player::P0;
    for (index, name) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .into_iter()
        .enumerate()
    {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(!game.object_view(land).tapped);
        game.state.players[player.index()].mana = [0; 6];
        let mut production = [0; 6];
        production[index] = 1;
        choose(&mut game, |action| {
            matches!(
                action, Action::ActivateMana { card, production: offered, .. }
                    if *card == land && *offered == production
            )
        });
        assert_eq!(game.state.players[player.index()].mana, production);
    }
}

#[test]
fn escape_tunnel_searches_for_a_tapped_basic_or_makes_a_small_creature_unblockable() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tunnel = put(&mut game, player, "Escape Tunnel", Zone::Battlefield);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let basic = put(&mut game, player, "Island", Zone::Library);

    let mut fetch = game.clone();
    activate(&mut fetch, tunnel, 0);
    pass_pair(&mut fetch);
    act(&mut fetch, Action::ChooseSearch { card: basic });
    if fetch.legal_actions().contains(&Action::FinishSearch) {
        act(&mut fetch, Action::FinishSearch);
    }
    assert!(fetch.on_battlefield(basic));
    assert!(fetch.object_view(basic).tapped);
    assert_eq!(fetch.object_view(tunnel).zone, Zone::Graveyard);

    announce_ability(&mut game, tunnel, 1);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(large),
        },
    );
    target(&mut game, small);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(tunnel).zone, Zone::Graveyard);
    assert!(game.keywords(small).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn magnifying_glass_taps_for_colorless_or_investigates_for_four() {
    let player = Player::P1;
    let mut game = scenario(player);
    let glass = put(&mut game, player, "Magnifying Glass", Zone::Battlefield);
    let mut produce = game.clone();
    produce.state.players[player.index()].mana = [0; 6];
    mana(&mut produce, glass, 0, COLORLESS);
    assert_eq!(produce.state.players[player.index()].mana, COLORLESS);
    assert!(produce.object_view(glass).tapped);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    activate(&mut game, glass, 1);
    assert!(game.object_view(glass).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    let token = game.controlled_token_ids(player).next().unwrap();
    assert_eq!(game.object_view(token).card, CLUE);
}

#[test]
fn sanitation_automaton_looks_at_one_card_and_can_send_it_to_the_graveyard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let automaton = put(&mut game, player, "Sanitation Automaton", Zone::Hand);
    let top = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, automaton, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut keep = game.clone();
    arrange(&mut keep, top, LibraryDestination::Top);
    assert_eq!(keep.top_library_card(player), Some(top));
    arrange(&mut game, top, LibraryDestination::Graveyard);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn gravestone_strider_filters_once_and_exiles_a_card_from_either_graveyard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let strider = put(&mut game, player, "Gravestone Strider", Zone::Battlefield);
    game.state.players[player.index()].mana = COLORLESS;
    let filter = Action::ActivateMana {
        card: strider,
        ability: 0,
        payment: COLORLESS,
        counter_payment: Counters::default(),
        production: BLUE,
    };
    act(&mut game, filter.clone());
    assert_eq!(game.state.players[player.index()].mana, BLUE);
    game.state.players[player.index()].mana = COLORLESS;
    reject(&mut game, filter);
    game.move_object(strider, Zone::Graveyard);
    let spell = put(&mut game, player.other(), "Shock", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    announce_ability(&mut game, strider, 1);
    target(&mut game, spell);
    pay(&mut game);
    assert_eq!(game.object_view(strider).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}
