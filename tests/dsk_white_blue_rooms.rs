//! Paid casts and unlocked-door rules for Duskmourn's white and blue Rooms.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;
const ELEVATOR: &str = "Central Elevator // Promising Stairs";
const GRAND_ENTRYWAY: &str = "Grand Entryway // Elegant Rotunda";
const MEAT_LOCKER: &str = "Meat Locker // Drowned Diner";
const UNDERWATER_TUNNEL: &str = "Underwater Tunnel // Slimy Aquarium";
const OPEN: RoomDoors = RoomDoors {
    left: true,
    right: true,
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn resolve_to_choice(game: &mut Game) {
    for _ in 0..24 {
        if game.state.pending != Pending::Priority || game.state.stack.is_empty() {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("Room did not reach its next choice");
}

fn cast_half(game: &mut Game, room: CardId, door: RoomDoor, name: &str, value: u64) {
    act(
        game,
        Action::AnnounceRoom {
            card: room,
            door,
            flash_surcharge: false,
        },
    );
    pay(game);
    assert_eq!(game.object_view(room).name, name);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), value);
    resolve_to_choice(game);
    let view = game.object_view(room);
    assert_eq!(view.zone, Zone::Battlefield);
    assert!(view.has_subtype("Room"));
    assert_eq!(view.name, name);
    assert_eq!(
        view.room_doors,
        Some(RoomDoors {
            left: door == RoomDoor::Left,
            right: door == RoomDoor::Right,
        })
    );
}

fn unlock(game: &mut Game, room: CardId, door: RoomDoor) {
    let exact = game.object_ref(room);
    choose(game, |action| {
        matches!(action, Action::BeginRoomUnlock {
            object, door: candidate, ..
        } if *object == exact && *candidate == door)
    });
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    assert_eq!(game.object_ref(room), exact);
    assert_eq!(game.object_view(room).room_doors, Some(OPEN));
    resolve_to_choice(game);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn targets(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        act(game, target(card));
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn begin_attacks(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn attack_player(game: &mut Game, cards: &[CardId]) {
    begin_attacks(game);
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn finish_combat(game: &mut Game) {
    for _ in 0..32 {
        if game.state.step == Step::Main2 && game.state.stack.is_empty() {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    panic!("unblocked Room combat did not reach the second main phase");
}

fn next_untap(game: &mut Game, player: Player) {
    game.state.step = Step::End;
    pass_pair(game);
    assert_eq!(game.state.active, player);
    assert_eq!(game.state.step, Step::Upkeep);
}

fn only_token(game: &Game, subtype: &str) -> ObjectView {
    let tokens: Vec<_> = game
        .observe(PLAYER)
        .objects
        .into_iter()
        .filter(|object| {
            object.is_token() && object.zone == Zone::Battlefield && object.has_subtype(subtype)
        })
        .collect();
    assert_eq!(tokens.len(), 1, "{subtype}");
    tokens.into_iter().next().unwrap()
}

#[test]
fn pool_returns_to_the_owner_and_locker_draws_once_per_combat_damage_batch() {
    let mut game = scenario(PLAYER);
    let room = put(
        &mut game,
        PLAYER,
        "Bottomless Pool // Locker Room",
        Zone::Hand,
    );
    let stolen = put(&mut game, OPPONENT, "Hill Giant", Zone::Battlefield);
    let stolen_ref = game.object_ref(stolen);
    game.record_control(stolen_ref, PLAYER, None);
    let double_striker = put(&mut game, PLAYER, "Raging Redcap", Zone::Battlefield);
    let bear = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    cast_half(&mut game, room, RoomDoor::Left, "Bottomless Pool", 1);
    let mut decline = game.clone();
    targets(&mut decline, &[]);
    settle(&mut decline);
    assert_eq!(decline.object_ref(stolen), stolen_ref);
    targets(&mut game, &[stolen]);
    settle(&mut game);
    let returned = game.object_view(stolen);
    assert_eq!((returned.zone, returned.owner), (Zone::Hand, OPPONENT));
    assert_ne!(returned.reference, stolen_ref);

    unlock(&mut game, room, RoomDoor::Right);
    attack_player(&mut game, &[double_striker, bear]);
    finish_combat(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.state.players[OPPONENT.index()].life, 16);
}

#[test]
fn central_elevator_reveals_only_a_found_room_with_no_shared_open_name() {
    let mut game = scenario(PLAYER);
    let room = put(&mut game, PLAYER, ELEVATOR, Zone::Hand);
    let open = put(&mut game, PLAYER, MEAT_LOCKER, Zone::Battlefield);
    game.open_room_on_entry(game.object_ref(open), RoomDoor::Left);
    let locked = put(&mut game, PLAYER, GRAND_ENTRYWAY, Zone::Battlefield);
    assert_eq!(game.object_view(locked).name, "");
    let blocked = put(&mut game, PLAYER, MEAT_LOCKER, Zone::Library);
    let found = put(&mut game, PLAYER, GRAND_ENTRYWAY, Zone::Library);
    cast_half(&mut game, room, RoomDoor::Left, "Central Elevator", 4);
    assert_eq!(game.state.pending, Pending::EffectSearch { player: PLAYER });
    assert!(game.observe(PLAYER).search_choice.is_some());
    assert!(game.observe(OPPONENT).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: blocked });

    let mut failed_search = game.clone();
    act(&mut failed_search, Action::FinishSearch);
    settle(&mut failed_search);
    assert_eq!(failed_search.object_view(found).zone, Zone::Library);
    let selection = Action::ChooseSearch { card: found };
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(PLAYER, indexed.revision(), index)
        .unwrap();
    act(&mut game, selection);
    assert_eq!(game.snapshot(), indexed.snapshot());
    game.restore(&checkpoint).unwrap();
    game.apply_index(PLAYER, game.revision(), index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    if game.legal_actions().contains(&Action::FinishSearch) {
        assert!(game.observe(OPPONENT).search_choice.is_none());
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
    assert!(
        game.observe(OPPONENT)
            .known_hand
            .contains(&game.object_ref(found))
    );
    assert_eq!(game.object_view(blocked).zone, Zone::Library);
}

#[test]
fn promising_stairs_surveils_before_checking_eight_distinct_unlocked_names() {
    let mut game = scenario(PLAYER);
    let stairs = put(&mut game, PLAYER, ELEVATOR, Zone::Hand);
    cast_half(&mut game, stairs, RoomDoor::Right, "Promising Stairs", 3);
    for name in [MEAT_LOCKER, GRAND_ENTRYWAY, UNDERWATER_TUNNEL] {
        let room = put(&mut game, PLAYER, name, Zone::Battlefield);
        game.set_room_doors(game.object_ref(room), OPEN);
    }
    let duplicate = put(&mut game, PLAYER, MEAT_LOCKER, Zone::Battlefield);
    game.open_room_on_entry(game.object_ref(duplicate), RoomDoor::Left);
    for should_win in [false, true] {
        let mut branch = game.clone();
        if should_win {
            unlock(&mut branch, stairs, RoomDoor::Left);
            act(&mut branch, Action::FinishSearch);
        }
        let top = *branch.state.players[PLAYER.index()].library.last().unwrap();
        branch.begin_step(Step::Upkeep);
        branch.prepare_triggers();
        resolve_to_choice(&mut branch);
        assert_eq!(branch.outcome(), None);
        let choice = branch.observe(PLAYER).library_choice.unwrap();
        assert!(choice.surveil);
        assert_eq!(choice.remaining.len(), 1);
        assert_eq!(choice.remaining[0].id, top);
        assert!(branch.observe(OPPONENT).library_choice.is_none());
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        assert_eq!(branch.object_view(top).zone, Zone::Graveyard);
        assert_eq!(
            branch.outcome(),
            should_win.then_some(Outcome::Winner(PLAYER))
        );
    }
}

#[test]
fn theater_grants_creature_convoke_and_prop_room_untaps_only_creatures_off_turn() {
    let mut game = scenario(PLAYER);
    let room = put(
        &mut game,
        PLAYER,
        "Dazzling Theater // Prop Room",
        Zone::Hand,
    );
    let donors =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, PLAYER, name, Zone::Battlefield));
    for id in donors {
        game.state.objects[id as usize].summoning_sick = true;
    }
    let spell = put(&mut game, PLAYER, "Grizzly Bears", Zone::Hand);
    let bow = put(&mut game, PLAYER, "Short Bow", Zone::Hand);
    game.state.players[PLAYER.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceCast { card: spell });
    game.state.players[PLAYER.index()].mana = [1, 0, 0, 0, 0, 3];
    cast_half(&mut game, room, RoomDoor::Left, "Dazzling Theater", 4);
    assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
    reject(&mut game, Action::AnnounceCast { card: bow });
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::BeginConvoke);
    for (card, contribution) in donors.into_iter().zip([
        convoke::ConvokeContribution::Green,
        convoke::ConvokeContribution::Generic,
    ]) {
        let object = game.object_ref(card);
        act(
            &mut game,
            Action::ChooseConvoker {
                object,
                contribution,
            },
        );
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert!(donors.into_iter().all(|id| game.object_view(id).tapped));

    game.add_counter(donors[1], CounterKind::Stun, 1);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    let mut locked = game.clone();
    next_untap(&mut locked, OPPONENT);
    assert!(locked.object_view(donors[0]).tapped);
    assert_eq!(locked.counter_count(donors[1], CounterKind::Stun), 1);
    game.state.players[PLAYER.index()].mana = [1, 0, 0, 0, 0, 2];
    unlock(&mut game, room, RoomDoor::Right);
    next_untap(&mut game, OPPONENT);
    assert!(!game.object_view(donors[0]).tapped);
    assert!(game.object_view(donors[1]).tapped);
    assert_eq!(game.counter_count(donors[1], CounterKind::Stun), 0);
    assert!(game.object_view(land).tapped);
}

#[test]
fn dollmaker_groups_non_toy_player_attacks_and_gallery_sets_live_base_stats() {
    let mut game = scenario(PLAYER);
    let room = put(
        &mut game,
        PLAYER,
        "Dollmaker's Shop // Porcelain Gallery",
        Zone::Hand,
    );
    let attackers =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, PLAYER, name, Zone::Battlefield));
    cast_half(&mut game, room, RoomDoor::Left, "Dollmaker's Shop", 2);

    let mut only_toys = game.clone();
    let toy = put(
        &mut only_toys,
        PLAYER,
        "Clockwork Percussionist",
        Zone::Battlefield,
    );
    attack_player(&mut only_toys, &[toy]);
    assert!(only_toys.state.stack.is_empty());
    let mut at_walker = game.clone();
    let walker = put(
        &mut at_walker,
        OPPONENT,
        "Chandra, Flameshaper",
        Zone::Battlefield,
    );
    at_walker.add_counter(walker, CounterKind::Loyalty, 4);
    begin_attacks(&mut at_walker);
    let defender = at_walker.object_ref(walker);
    act(
        &mut at_walker,
        Action::AttackPlaneswalker {
            card: attackers[0],
            defender,
        },
    );
    act(&mut at_walker, Action::FinishAttackers);
    assert!(at_walker.state.stack.is_empty());

    attack_player(&mut game, &attackers);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let toy = only_token(&game, "Toy");
    assert_eq!((toy.name, toy.kinds, toy.colors), ("Toy Token", 17, 1));
    assert_eq!((toy.power, toy.toughness), (1, 1));
    finish_combat(&mut game);
    game.state.players[PLAYER.index()].mana = [2, 0, 0, 0, 0, 4];
    unlock(&mut game, room, RoomDoor::Right);
    assert_eq!(stats(&game, toy.id), (3, 3));
    game.add_counter(attackers[0], CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, attackers[0]), (4, 4));
    game.move_object(attackers[1], Zone::Graveyard);
    assert_eq!(stats(&game, toy.id), (2, 2));
    assert_eq!(stats(&game, attackers[0]), (3, 3));
}

#[test]
fn grand_entryway_creates_a_glimmer_and_rotunda_keeps_each_legal_target() {
    let mut game = scenario(PLAYER);
    let room = put(&mut game, PLAYER, GRAND_ENTRYWAY, Zone::Hand);
    let own = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, OPPONENT, "Healer's Hawk", Zone::Battlefield);
    let bounce = put(&mut game, PLAYER, "Unsummon", Zone::Hand);
    cast_half(&mut game, room, RoomDoor::Left, "Grand Entryway", 2);
    let glimmer = only_token(&game, "Glimmer");
    assert_eq!(
        (glimmer.name, glimmer.kinds, glimmer.colors),
        ("Glimmer Token", 33, 1)
    );
    assert_eq!((glimmer.power, glimmer.toughness), (1, 1));

    unlock(&mut game, room, RoomDoor::Right);
    let mut decline = game.clone();
    targets(&mut decline, &[]);
    settle(&mut decline);
    assert_eq!(decline.counter_count(own, CounterKind::PlusOne), 0);
    assert_eq!(decline.counter_count(foreign, CounterKind::PlusOne), 0);
    act(&mut game, target(own));
    reject(&mut game, target(own));
    targets(&mut game, &[foreign]);
    cast(&mut game, bounce, Some(Target::Card(own)));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(glimmer.id, CounterKind::PlusOne), 0);
}

#[test]
fn meat_locker_stuns_an_already_tapped_creature_and_diner_draws_before_discard() {
    let mut game = scenario(PLAYER);
    let room = put(&mut game, PLAYER, MEAT_LOCKER, Zone::Hand);
    let creature = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    cast_half(&mut game, room, RoomDoor::Left, "Meat Locker", 3);
    targets(&mut game, &[creature]);
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::Stun), 2);
    assert!(game.object_view(creature).tapped);

    unlock(&mut game, room, RoomDoor::Right);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { player: PLAYER, .. }
    ));
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 3);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert!(
        game.observe(OPPONENT)
            .objects
            .iter()
            .all(|object| object.id != top)
    );
    act(&mut game, Action::Discard { card: top });
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    for (remaining, tapped) in [(1, true), (0, true), (0, false)] {
        next_untap(&mut game, OPPONENT);
        next_untap(&mut game, PLAYER);
        assert_eq!(game.counter_count(creature, CounterKind::Stun), remaining);
        assert_eq!(game.object_view(creature).tapped, tapped);
    }
}

#[test]
fn mirror_room_copies_independent_targets_and_realm_only_multiplies_your_permanents() {
    let mut game = scenario(PLAYER);
    let room = put(
        &mut game,
        PLAYER,
        "Mirror Room // Fractured Realm",
        Zone::Hand,
    );
    let hawk = put(&mut game, PLAYER, "Healer's Hawk", Zone::Battlefield);
    let giada = put(&mut game, PLAYER, "Giada, Font of Hope", Zone::Battlefield);
    let opponent = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    cast_half(&mut game, room, RoomDoor::Right, "Fractured Realm", 7);
    unlock(&mut game, room, RoomDoor::Left);
    assert_eq!(
        game.state.pending,
        Pending::OrderTriggers { player: PLAYER }
    );
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    reject(&mut game, target(opponent));
    targets(&mut game, &[hawk]);
    targets(&mut game, &[giada]);
    assert_eq!(game.state.stack.len(), 2);

    resolve_to_choice(&mut game);
    assert_eq!(game.state.pending, Pending::Legend { player: PLAYER });
    let giada_copy = only_token(&game, "Angel");
    assert_eq!(giada_copy.name, "Giada, Font of Hope");
    assert!(giada_copy.is_legendary() && giada_copy.has_subtype("Reflection"));
    assert!(giada_copy.keywords.contains(Keywords::FLYING));
    assert!(!giada_copy.keywords.contains(Keywords::HASTE));
    act(
        &mut game,
        Action::KeepLegend {
            object: giada_copy.reference,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(giada).zone, Zone::Graveyard);
    let hawk_copy = only_token(&game, "Bird");
    assert_eq!(hawk_copy.name, "Healer's Hawk");
    assert!(hawk_copy.has_subtype("Reflection"));
    assert!(hawk_copy.keywords.contains(Keywords::LIFELINK));

    for (player, expected) in [(PLAYER, 2), (OPPONENT, 1)] {
        if player == OPPONENT {
            next_untap(&mut game, OPPONENT);
            game.state.step = Step::Main1;
        }
        game.state.players[player.index()].mana = [3; 6];
        let overseer = put(&mut game, player, "Inspiring Overseer", Zone::Hand);
        cast(&mut game, overseer, None);
        resolve_to_choice(&mut game);
        if player == PLAYER {
            assert_eq!(game.state.pending, Pending::OrderTriggers { player });
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            expected
        );
    }
}

#[test]
fn surgical_suite_checks_graveyard_and_mana_value_and_hospital_triggers_once() {
    let mut game = scenario(PLAYER);
    let room = put(
        &mut game,
        PLAYER,
        "Surgical Suite // Hospital Room",
        Zone::Hand,
    );
    let three = put(&mut game, PLAYER, "Inspiring Overseer", Zone::Graveyard);
    let four = put(&mut game, PLAYER, "Hill Giant", Zone::Graveyard);
    let foreign = put(&mut game, OPPONENT, "Bear Cub", Zone::Graveyard);
    let attackers =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, PLAYER, name, Zone::Battlefield));
    cast_half(&mut game, room, RoomDoor::Left, "Surgical Suite", 2);
    reject(&mut game, target(four));
    reject(&mut game, target(foreign));
    targets(&mut game, &[three]);
    settle(&mut game);
    assert_eq!(game.object_view(three).zone, Zone::Battlefield);
    assert_eq!(game.object_view(three).controller, PLAYER);
    assert_eq!(game.object_view(four).zone, Zone::Graveyard);

    unlock(&mut game, room, RoomDoor::Right);
    attack_player(&mut game, &attackers);
    reject(&mut game, target(three));
    targets(&mut game, &[attackers[1]]);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(attackers[0], CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(attackers[1], CounterKind::PlusOne), 1);
}

#[test]
fn tunnel_surveils_privately_and_aquarium_counters_only_the_exact_manifest() {
    let mut game = scenario(PLAYER);
    let room = put(&mut game, PLAYER, UNDERWATER_TUNNEL, Zone::Hand);
    let bystander = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let elves = put(&mut game, PLAYER, "Llanowar Elves", Zone::Library);
    let shock = put(&mut game, PLAYER, "Shock", Zone::Library);
    cast_half(&mut game, room, RoomDoor::Left, "Underwater Tunnel", 1);
    let choice = game.observe(PLAYER).library_choice.unwrap();
    assert!(choice.surveil);
    assert_eq!(choice.remaining.len(), 2);
    assert!(game.observe(OPPONENT).library_choice.is_none());
    for (card, destination) in [
        (shock, LibraryDestination::Graveyard),
        (elves, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);

    let mut empty = game.clone();
    for card in empty.state.players[PLAYER.index()].library.clone() {
        empty.move_object(card, Zone::Exile);
    }
    unlock(&mut empty, room, RoomDoor::Right);
    settle(&mut empty);
    assert!(empty.observe(PLAYER).face_down_permanents.is_empty());
    assert_eq!(empty.counter_count(bystander, CounterKind::PlusOne), 0);

    unlock(&mut game, room, RoomDoor::Right);
    let choice = game.observe(PLAYER).manifest_choice.unwrap();
    assert_eq!(choice.candidates.len(), 2);
    assert!(game.observe(OPPONENT).manifest_choice.is_none());
    let object = game.object_ref(elves);
    act(&mut game, Action::ChooseManifest { object });
    let manifested = game.object_view(elves);
    assert_eq!(manifested.name, "");
    assert_eq!(stats(&game, elves), (3, 3));
    assert_eq!(game.counter_count(bystander, CounterKind::PlusOne), 0);
    assert_eq!(
        game.observe(OPPONENT).face_down_permanents[0].underlying,
        None
    );
    choose(
        &mut game,
        |action| matches!(action, Action::BeginFaceUp { object, .. } if *object == manifested.reference),
    );
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.object_ref(elves), manifested.reference);
    assert_eq!(game.object_view(elves).name, "Llanowar Elves");
    assert_eq!(game.counter_count(elves, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, elves), (2, 2));
}
