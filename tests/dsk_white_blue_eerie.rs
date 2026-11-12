//! Actual enchantment entry, Eerie, and Room door interactions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;
const POOL: &str = "Bottomless Pool // Locker Room";
const OPEN: RoomDoors = RoomDoors {
    left: true,
    right: true,
};
const LEFT_ONLY: RoomDoors = RoomDoors {
    left: true,
    right: false,
};

fn resolve_to_choice(game: &mut Game) {
    for _ in 0..24 {
        if game.state.pending != Pending::Priority || game.state.stack.is_empty() {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("Eerie scenario did not reach its next choice");
}

fn resolve_untargeted(game: &mut Game) {
    for _ in 0..40 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishTargets | Action::PutTrigger { .. }
            )
        });
    }
    panic!("Eerie scenario did not settle");
}

fn pay_door(game: &mut Game, room: CardId, door: RoomDoor) {
    let object = game.object_ref(room);
    choose(game, |action| {
        matches!(action, Action::BeginRoomUnlock {
            object: candidate, door: side, ..
        } if *candidate == object && *side == door)
    });
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

fn creatures_with_subtype(game: &Game, subtype: &str) -> Vec<ObjectView> {
    game.observe(PLAYER)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == PLAYER
                && object.is_token()
                && object.has_subtype(subtype)
        })
        .collect()
}

fn assert_glimmer(glimmer: &ObjectView) {
    assert_eq!(
        (glimmer.name, glimmer.kinds, glimmer.colors),
        ("Glimmer Token", 33, 1)
    );
    assert_eq!((glimmer.power, glimmer.toughness), (1, 1));
}

fn begin_attacks(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn end_turn(game: &mut Game) {
    game.state.step = Step::End;
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn cult_healer_apparition_and_researcher_react_to_entry_and_only_the_last_door() {
    let mut game = scenario(PLAYER);
    let healer = put(&mut game, PLAYER, "Cult Healer", Zone::Battlefield);
    let apparition = put(&mut game, PLAYER, "Erratic Apparition", Zone::Battlefield);
    let researcher = put(&mut game, PLAYER, "Stalked Researcher", Zone::Battlefield);
    let room = put(&mut game, PLAYER, POOL, Zone::Battlefield);
    pay_door(&mut game, room, RoomDoor::Left);
    resolve_untargeted(&mut game);
    assert_eq!(game.object_view(room).room_doors, Some(LEFT_ONLY));
    assert_eq!(stats(&game, apparition), (1, 3));
    assert!(
        !game
            .object_view(healer)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    let mut before = game.clone();
    begin_attacks(&mut before);
    reject(&mut before, Action::Attack { card: researcher });

    let surveyor = put(&mut game, PLAYER, "Tunnel Surveyor", Zone::Hand);
    cast(&mut game, surveyor, None);
    resolve_untargeted(&mut game);
    assert_eq!(stats(&game, surveyor), (2, 2));
    let glimmers = creatures_with_subtype(&game, "Glimmer");
    assert_eq!(glimmers.len(), 1);
    assert_glimmer(&glimmers[0]);
    assert_eq!(stats(&game, apparition), (2, 4));
    assert!(
        game.object_view(healer)
            .keywords
            .contains(Keywords::LIFELINK)
    );

    pay_door(&mut game, room, RoomDoor::Right);
    resolve_untargeted(&mut game);
    assert_eq!(game.object_view(room).room_doors, Some(OPEN));
    assert_eq!(stats(&game, apparition), (3, 5));
    assert!(
        game.object_view(researcher)
            .keywords
            .contains(Keywords::DEFENDER)
    );
    assert!(
        game.object_view(apparition)
            .keywords
            .contains(Keywords::FLYING)
    );
    begin_attacks(&mut game);
    for card in [healer, apparition, researcher] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    assert!(!game.object_view(apparition).tapped);
    assert!(game.object_view(healer).tapped);
    for _ in 0..24 {
        if game.state.step == Step::Main2 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    assert_eq!(game.state.players[PLAYER.index()].life, 23);
    assert_eq!(game.state.players[OPPONENT.index()].life, 11);
    end_turn(&mut game);
    assert_eq!(game.state.active, OPPONENT);
    assert_eq!(stats(&game, apparition), (1, 3));
    assert!(
        !game
            .object_view(healer)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    end_turn(&mut game);
    begin_attacks(&mut game);
    reject(&mut game, Action::Attack { card: researcher });
}

#[test]
fn entity_tracker_flashes_on_the_opponents_turn_and_only_your_glimmer_counts() {
    let mut game = scenario(OPPONENT);
    let tracker = put(&mut game, PLAYER, "Entity Tracker", Zone::Hand);
    let opposing_enchantment = put(&mut game, OPPONENT, "Omniscience", Zone::Hand);
    let burst = put(&mut game, PLAYER, "Glimmerburst", Zone::Hand);
    cast(&mut game, opposing_enchantment, None);
    act(&mut game, Action::Pass);
    cast(&mut game, tracker, None);
    settle(&mut game);
    assert_eq!(stats(&game, tracker), (2, 3));
    assert!(game.object_view(tracker).keywords.contains(Keywords::FLASH));
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 0);
    if game.actor() != Some(PLAYER) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, burst, None);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 3);
    let glimmers = creatures_with_subtype(&game, "Glimmer");
    assert_eq!(glimmers.len(), 1);
    assert_glimmer(&glimmers[0]);
}

#[test]
fn optimistic_scavenger_targets_either_controllers_creature_on_each_eerie_event() {
    let mut game = scenario(PLAYER);
    let scavenger = put(&mut game, PLAYER, "Optimistic Scavenger", Zone::Battlefield);
    let opposing = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let room = put(&mut game, PLAYER, POOL, Zone::Battlefield);
    game.open_room_on_entry(game.object_ref(room), RoomDoor::Left);
    let burst = put(&mut game, PLAYER, "Glimmerburst", Zone::Hand);
    cast(&mut game, burst, None);
    resolve_to_choice(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(room),
        },
    );
    target(&mut game, Target::Card(opposing));
    settle(&mut game);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(scavenger, CounterKind::PlusOne), 0);
    pay_door(&mut game, room, RoomDoor::Right);
    target(&mut game, Target::Card(scavenger));
    settle(&mut game);
    assert_eq!(stats(&game, scavenger), (2, 2));
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);
}

#[test]
fn scrabbling_skullcrab_mills_two_for_each_chosen_player() {
    let mut game = scenario(PLAYER);
    let crab = put(&mut game, PLAYER, "Scrabbling Skullcrab", Zone::Battlefield);
    let room = put(&mut game, PLAYER, POOL, Zone::Battlefield);
    game.open_room_on_entry(game.object_ref(room), RoomDoor::Left);
    let surveyor = put(&mut game, PLAYER, "Tunnel Surveyor", Zone::Hand);
    cast(&mut game, surveyor, None);
    resolve_to_choice(&mut game);
    assert_eq!(stats(&game, crab), (0, 3));
    for player in [PLAYER, OPPONENT] {
        if player == OPPONENT {
            pay_door(&mut game, room, RoomDoor::Right);
        }
        let library = &game.state.players[player.index()].library;
        let before = library.len();
        let milled = [library[before - 1], library[before - 2]];
        target(&mut game, Target::Player(player));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].library.len(), before - 2);
        assert!(
            milled
                .into_iter()
                .all(|card| game.object_view(card).zone == Zone::Graveyard)
        );
    }
}

#[test]
fn unwilling_vessel_sizes_its_flying_spirit_from_all_departing_counters() {
    let mut game = scenario(PLAYER);
    let vessel = put(&mut game, PLAYER, "Unwilling Vessel", Zone::Battlefield);
    let room = put(&mut game, PLAYER, POOL, Zone::Battlefield);
    game.open_room_on_entry(game.object_ref(room), RoomDoor::Left);
    let surveyor = put(&mut game, PLAYER, "Tunnel Surveyor", Zone::Hand);
    let removal = put(&mut game, PLAYER, "Murder", Zone::Hand);
    cast(&mut game, surveyor, None);
    settle(&mut game);
    assert!(
        game.object_view(vessel)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert_eq!(game.counter_count(vessel, CounterKind::Possession), 1);
    pay_door(&mut game, room, RoomDoor::Right);
    settle(&mut game);
    assert_eq!(game.counter_count(vessel, CounterKind::Possession), 2);
    game.add_counter(vessel, CounterKind::PlusOne, 1);
    let original = game.object_ref(vessel);
    cast(&mut game, removal, Some(Target::Card(vessel)));
    settle(&mut game);
    assert_eq!(game.object_view(vessel).zone, Zone::Graveyard);
    assert_ne!(game.object_ref(vessel), original);
    let spirits = creatures_with_subtype(&game, "Spirit");
    assert_eq!(spirits.len(), 1);
    let spirit = &spirits[0];
    assert_eq!(
        (spirit.name, spirit.kinds, spirit.colors),
        ("Spirit Token", 1, 2)
    );
    assert!(spirit.keywords.contains(Keywords::FLYING));
    assert_eq!((spirit.power, spirit.toughness), (3, 3));
}

#[test]
fn ghostly_keybearer_unlocks_an_exact_controlled_room_after_player_combat_damage() {
    let mut game = scenario(PLAYER);
    let keybearer = put(&mut game, PLAYER, "Ghostly Keybearer", Zone::Battlefield);
    let room = put(&mut game, PLAYER, POOL, Zone::Battlefield);
    let already_open = put(
        &mut game,
        PLAYER,
        "Meat Locker // Drowned Diner",
        Zone::Battlefield,
    );
    game.set_room_doors(game.object_ref(already_open), OPEN);
    let foreign = put(&mut game, OPPONENT, POOL, Zone::Battlefield);
    assert!(
        game.object_view(keybearer)
            .keywords
            .contains(Keywords::FLYING)
    );
    begin_attacks(&mut game);
    act(&mut game, Action::Attack { card: keybearer });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..20 {
        if matches!(game.state.pending, Pending::Targets { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.pending, Pending::Targets { player: PLAYER });
    assert_eq!(game.state.players[OPPONENT.index()].life, 17);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    let mut open_target = game.clone();
    target(&mut open_target, Target::Card(already_open));
    settle(&mut open_target);
    assert_eq!(
        open_target.object_view(room).room_doors,
        Some(RoomDoors::default())
    );

    target(&mut game, Target::Card(room));
    resolve_to_choice(&mut game);
    assert_eq!(game.state.pending, Pending::RoomChoice { player: PLAYER });
    let object = game.object_ref(room);
    let opposite = game.object_ref(foreign);
    reject(
        &mut game,
        Action::ChooseRoomDoor {
            object: opposite,
            door: RoomDoor::Left,
            change: RoomChange::Unlock,
        },
    );
    assert!(game.legal_actions().contains(&Action::ChooseRoomDoor {
        object,
        door: RoomDoor::Right,
        change: RoomChange::Unlock,
    }));
    act(
        &mut game,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Left,
            change: RoomChange::Unlock,
        },
    );
    resolve_untargeted(&mut game);
    assert_eq!(game.object_ref(room), object);
    assert_eq!(game.object_view(room).room_doors, Some(LEFT_ONLY));
}
