//! Paid scenarios for green and multicolored Duskmourn Rooms and their interactions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SMOKY_ROOM: &str = "Smoky Lounge // Misty Salon";

fn progress_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario exceeded its action budget");
}

fn settle_ordered(game: &mut Game) {
    progress_until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
    });
}

fn next_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    progress_until(game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1
    });
}

fn cast_named(game: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, target);
    card
}

fn cast_room(game: &mut Game, card: CardId, door: RoomDoor) {
    act(
        game,
        Action::AnnounceRoom {
            card,
            door,
            flash_surcharge: false,
        },
    );
    pay(game);
}

fn unlock(game: &mut Game, card: CardId, door: RoomDoor, cost: ManaCost) {
    let object = game.object_ref(card);
    act(game, Action::BeginRoomUnlock { object, door, cost });
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn finish_targets_and_pay(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn manifest(game: &mut Game, player: Player, card: CardId) {
    progress_until(game, |game| game.observe(player).manifest_choice.is_some());
    let object = game.object_ref(card);
    act(game, Action::ChooseManifest { object });
}

#[test]
fn moldering_gym_searches_and_weight_room_counters_only_the_new_manifest() {
    let player = Player::P0;
    let mut game = scenario(player);
    let found = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = put(&mut game, player, "Thornwood Falls", Zone::Library);
    let room = put(
        &mut game,
        player,
        "Moldering Gym // Weight Room",
        Zone::Hand,
    );
    cast_room(&mut game, room, RoomDoor::Left);
    progress_until(&mut game, |game| {
        game.observe(player).search_choice.is_some()
    });
    assert!(game.observe(player.other()).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Battlefield);
    assert!(game.object_view(found).tapped);

    put(&mut game, player, "Shock", Zone::Library);
    let old = put(&mut game, player, "Llanowar Elves", Zone::Library);
    cast_named(&mut game, player, "Manifest Dread", None);
    manifest(&mut game, player, old);
    settle(&mut game);
    let milled = put(&mut game, player, "Shock", Zone::Library);
    let new = put(&mut game, player, "Bear Cub", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 5];
    unlock(
        &mut game,
        room,
        RoomDoor::Right,
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 5,
        },
    );
    manifest(&mut game, player, new);
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(stats(&game, new), (5, 5));
    assert_eq!(game.counter_count(new, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(old, CounterKind::PlusOne), 0);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn roaring_furnace_counts_live_hand_size_and_sauna_draws_without_a_hand_limit() {
    let player = Player::P1;
    let mut game = scenario(player);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let friendly = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Hand);
    put(&mut game, player, "Island", Zone::Hand);
    let room = put(
        &mut game,
        player,
        "Roaring Furnace // Steaming Sauna",
        Zone::Hand,
    );
    cast_room(&mut game, room, RoomDoor::Left);
    pass_pair(&mut game);
    reject(&mut game, target(friendly));
    act(&mut game, target(enemy));
    let mut unopposed = game.clone();
    settle(&mut unopposed);
    assert_eq!(unopposed.object_view(enemy).zone, Zone::Graveyard);
    cast(&mut game, growth, Some(Target::Card(friendly)));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy).damage, 2);
    assert!(!game.observe(player).players[player.index()].no_maximum_hand_size);

    let top = put(&mut game, player, "Mountain", Zone::Library);
    let mut closed = game.clone();
    progress_until(&mut closed, |game| game.state.step == Step::End);
    settle(&mut closed);
    assert_eq!(closed.object_view(top).zone, Zone::Library);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 3];
    unlock(
        &mut game,
        room,
        RoomDoor::Right,
        ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 3,
        },
    );
    assert!(game.observe(player).players[player.index()].no_maximum_hand_size);
    progress_until(&mut game, |game| game.state.step == Step::End);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
}

#[test]
fn closet_allows_one_graveyard_land_and_cellar_spells_survive_its_departure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let room = put(
        &mut game,
        player,
        "Walk-In Closet // Forgotten Cellar",
        Zone::Hand,
    );
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let another = put(&mut game, player, "Island", Zone::Graveyard);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let later = put(&mut game, player, "Shock", Zone::Graveyard);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast_room(&mut game, room, RoomDoor::Left);
    settle(&mut game);
    act(&mut game, Action::PlayLand { card: land });
    reject(&mut game, Action::PlayLand { card: another });
    reject(&mut game, Action::AnnounceCast { card: shock });

    unlock(
        &mut game,
        room,
        RoomDoor::Right,
        ManaCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 3,
        },
    );
    settle(&mut game);
    cast_named(&mut game, player, "Disenchant", Some(Target::Card(room)));
    settle(&mut game);
    assert_eq!(game.object_view(room).zone, Zone::Exile);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        18
    );
    cast_named(&mut game, player, "Murder", Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.observe(player).creatures_died_this_turn, 0);

    next_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: later });
    reject(&mut game, Action::PlayLand { card: another });
}

#[test]
fn anthropede_pays_either_cost_before_selecting_the_room_for_its_reflexive_trigger() {
    let player = Player::P1;
    let mut game = scenario(player);
    let room = put(&mut game, player.other(), SMOKY_ROOM, Zone::Battlefield);
    let ordinary = put(
        &mut game,
        player.other(),
        "Doubling Season",
        Zone::Battlefield,
    );
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 5];
    let source = cast_named(&mut game, player, "Anthropede", None);
    progress_until(&mut game, |game| {
        game.observe(player).resolution_payment.is_some()
    });
    assert!(game.object_view(source).keywords.contains(Keywords::REACH));
    reject(&mut game, target(room));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(room).zone, Zone::Battlefield);

    let mut mana = game.clone();
    act(&mut mana, Action::ChooseResolutionCost { index: 1 });
    act(
        &mut mana,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    assert_eq!(mana.state.players[player.index()].mana, [0; 6]);
    reject(&mut mana, target(ordinary));
    act(&mut mana, target(room));
    settle(&mut mana);
    assert_eq!(mana.object_view(room).zone, Zone::Graveyard);
    assert_eq!(mana.object_view(discarded).zone, Zone::Hand);

    act(&mut game, Action::ChooseResolutionCost { index: 0 });
    let object = game.object_ref(discarded);
    act(&mut game, Action::ChooseAdditionalCost { object });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.object_view(room).zone, Zone::Battlefield);
    act(&mut game, target(room));
    settle(&mut game);
    assert_eq!(game.object_view(room).zone, Zone::Graveyard);
}

#[test]
fn soulrager_sacrifices_an_owned_locked_room_and_resolves_after_its_own_death() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let room = put(&mut game, player, SMOKY_ROOM, Zone::Battlefield);
    let foreign = put(&mut game, opponent, SMOKY_ROOM, Zone::Battlefield);
    let nonroom = put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let source = cast_named(&mut game, player, "Intruding Soulrager", None);
    settle(&mut game);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors::default())
    );
    next_main(&mut game, player);
    progress_until(&mut game, |game| game.state.step == Step::Attackers);
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    assert!(!game.object_view(source).tapped);
    let drawn = put(&mut game, player, "Island", Zone::Library);
    act(&mut game, activation(source));
    for id in [foreign, nonroom] {
        let object = game.object_ref(id);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    let object = game.object_ref(room);
    act(&mut game, Action::ChooseAdditionalCost { object });
    finish_targets_and_pay(&mut game);
    assert_eq!(game.object_view(room).zone, Zone::Graveyard);
    assert!(game.object_view(source).tapped);
    assert!(game.state.stack.last().unwrap().targets.is_empty());

    act(&mut game, Action::Pass);
    game.state.players[opponent.index()].mana = [0, 0, 2, 0, 0, 1];
    cast_named(&mut game, opponent, "Murder", Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).players[opponent.index()].life, 18);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
}

#[test]
fn marina_publicly_takes_every_revealed_enchantment_and_changes_a_controlled_door() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let room = put(&mut game, player, SMOKY_ROOM, Zone::Library);
    let creature = put(&mut game, player, "Enduring Vitality", Zone::Library);
    let misses = ["Island", "Forest", "Shock", "Bear Cub", "Short Bow"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    let enemy_room = put(&mut game, opponent, SMOKY_ROOM, Zone::Battlefield);
    let source = cast_named(&mut game, player, "Marina Vendrell", None);
    progress_until(&mut game, |game| {
        game.observe(player).library_selection.is_some()
    });
    let public = game.observe(opponent).library_selection.unwrap();
    assert_eq!(public.remaining.len(), 7);
    reject(&mut game, Action::FinishLibrarySelection);
    reject(&mut game, Action::SelectLibrary { card: misses[0] });
    for card in [room, creature] {
        act(&mut game, Action::SelectLibrary { card });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(room).zone, Zone::Hand);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert!(
        game.observe(opponent)
            .known_hand
            .contains(&game.object_ref(room))
    );
    assert!(
        game.observe(opponent)
            .known_hand
            .contains(&game.object_ref(creature))
    );
    assert!(
        misses
            .iter()
            .all(|card| game.state.players[player.index()].library[..5].contains(card))
    );

    cast_room(&mut game, room, RoomDoor::Left);
    settle(&mut game);
    next_main(&mut game, player);
    settle(&mut game);
    act(&mut game, activation(source));
    reject(&mut game, target(enemy_room));
    act(&mut game, target(room));
    finish_targets_and_pay(&mut game);
    assert!(game.object_view(source).tapped);
    progress_until(&mut game, |game| game.observe(player).room_choice.is_some());
    let object = game.object_ref(room);
    let mut locked = game.clone();
    act(
        &mut locked,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Left,
            change: RoomChange::Lock,
        },
    );
    settle(&mut locked);
    assert_eq!(
        locked.object_view(room).room_doors,
        Some(RoomDoors::default())
    );
    act(
        &mut game,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Right,
            change: RoomChange::Unlock,
        },
    );
    settle(&mut game);
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors {
            left: true,
            right: true
        })
    );
}

#[test]
fn skullsnap_surveils_once_on_entry_and_once_when_the_room_becomes_fully_unlocked() {
    let player = Player::P0;
    let mut game = scenario(player);
    let skullsnap = cast_named(&mut game, player, "Skullsnap Nuisance", None);
    settle(&mut game);
    assert!(
        game.object_view(skullsnap)
            .keywords
            .contains(Keywords::FLYING)
    );
    let room = put(&mut game, player, SMOKY_ROOM, Zone::Hand);
    let first = put(&mut game, player, "Island", Zone::Library);
    cast_room(&mut game, room, RoomDoor::Left);
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    let choice = game.observe(player).library_choice.unwrap();
    assert_eq!(choice.remaining.len(), 1);
    assert!(choice.surveil);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: first,
            destination: LibraryDestination::Graveyard,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    let second = put(&mut game, player, "Forest", Zone::Library);
    unlock(
        &mut game,
        room,
        RoomDoor::Right,
        ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
        },
    );
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    assert_eq!(
        game.observe(player).library_choice.unwrap().remaining[0].id,
        second
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: second,
            destination: LibraryDestination::Top,
        },
    );
    settle_ordered(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&second)
    );
}
