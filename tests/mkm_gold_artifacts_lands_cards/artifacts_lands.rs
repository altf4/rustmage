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
    assert_eq!(game.object_view(bow).attached_to, Some(snuffler));
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
    assert_eq!(game.object_view(cap).attached_to, Some(bear));
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
