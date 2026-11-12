//! Paid scenarios for three green and multicolored Duskmourn deck cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

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

fn next_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    progress_until(game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1
    });
}

fn attack(game: &mut Game, card: CardId) {
    progress_until(game, |game| game.state.step == Step::Attackers);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn cast_named(game: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, target);
    card
}

#[test]
fn arabella_samples_signed_controlled_power_at_resolution_for_both_amounts() {
    let player = Player::P0;
    let mut game = scenario(player);
    let arabella = cast_named(&mut game, player, "Arabella, Abandoned Doll", None);
    settle(&mut game);
    assert_eq!(stats(&game, arabella), (1, 3));
    next_main(&mut game, player);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let negative = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[negative as usize].growth.power = -4;
    assert_eq!(stats(&game, negative).0, -1);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    attack(&mut game, arabella);
    assert_eq!(game.state.stack.len(), 1);
    let mut baseline = game.clone();
    settle(&mut baseline);
    let life = |game: &Game, seat: Player| game.observe(seat).players[seat.index()].life;
    assert_eq!(
        (life(&baseline, player), life(&baseline, player.other())),
        (23, 17)
    );

    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    cast_named(&mut game, player, "Giant Growth", Some(Target::Card(small)));
    settle(&mut game);
    assert_eq!(stats(&game, small).0, 5);
    assert_eq!((life(&game, player), life(&game, player.other())), (22, 18));
}

fn manifest(game: &mut Game, player: Player, card: CardId) {
    progress_until(game, |game| game.observe(player).manifest_choice.is_some());
    let object = game.object_ref(card);
    act(game, Action::ChooseManifest { object });
}

fn announce_reveal(game: &mut Game, source: CardId, player: Player) {
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn choose_reveal_target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn underlying(game: &Game, player: Player, card: CardId) -> Option<Card> {
    game.observe(player)
        .face_down_permanents
        .into_iter()
        .find(|permanent| permanent.object == game.object_ref(card))
        .unwrap()
        .underlying
}

#[test]
fn shrieker_reveals_both_kinds_turns_an_opponents_creature_up_and_manifests_on_attack() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let shrieker = cast_named(&mut game, player, "Hauntwoods Shrieker", None);
    settle(&mut game);
    put(&mut game, opponent, "Forest", Zone::Library);
    let instant = put(&mut game, opponent, "Shock", Zone::Library);
    act(&mut game, Action::Pass);
    cast_named(&mut game, opponent, "Growing Dread", None);
    manifest(&mut game, opponent, instant);
    settle(&mut game);
    assert_eq!(underlying(&game, player, instant), None);
    announce_reveal(&mut game, shrieker, player);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(shrieker),
        },
    );
    choose_reveal_target(&mut game, instant);
    settle(&mut game);
    assert_eq!(underlying(&game, player, instant), Card::from_name("Shock"));
    assert!(game.is_face_down(game.object_ref(instant)));

    next_main(&mut game, opponent);
    let miss = put(&mut game, opponent, "Forest", Zone::Library);
    let giant = put(&mut game, opponent, "Hill Giant", Zone::Library);
    game.state.players[opponent.index()].mana = [0, 0, 0, 0, 1, 1];
    cast_named(&mut game, opponent, "Manifest Dread", None);
    manifest(&mut game, opponent, giant);
    settle(&mut game);
    assert_eq!(game.object_view(miss).zone, Zone::Graveyard);
    let facedown = game.object_ref(giant);
    game.add_counter(giant, CounterKind::PlusOne, 1);
    act(&mut game, Action::Pass);
    announce_reveal(&mut game, shrieker, player);
    choose_reveal_target(&mut game, giant);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectOptional { .. })
    });
    assert_eq!(game.actor(), Some(player));
    assert_eq!(
        underlying(&game, player, giant),
        Card::from_name("Hill Giant")
    );
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert!(declined.is_face_down(facedown));
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_ref(giant), facedown);
    assert!(!game.is_face_down(facedown));
    assert_eq!(game.object_view(giant).controller, opponent);
    assert_eq!(stats(&game, giant), (4, 4));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    next_main(&mut game, player);
    let milled = put(&mut game, player, "Forest", Zone::Library);
    let found = put(&mut game, player, "Bear Cub", Zone::Library);
    attack(&mut game, shrieker);
    manifest(&mut game, player, found);
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert!(game.is_face_down(game.object_ref(found)));
}

#[test]
fn glimmers_reduce_enchantment_and_door_costs_without_reducing_colored_symbols() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = cast_named(&mut game, player, "Inquisitive Glimmer", None);
    settle(&mut game);
    cast_named(&mut game, player, "Inquisitive Glimmer", None);
    settle(&mut game);
    assert_eq!(stats(&game, first), (2, 3));

    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    reject(&mut game, Action::AnnounceCast { card: creature });
    let room = put(&mut game, player, "Smoky Lounge // Misty Salon", Zone::Hand);
    let announce = Action::AnnounceRoom {
        card: room,
        door: RoomDoor::Left,
        flash_surcharge: false,
    };
    reject(&mut game, announce.clone());
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, announce);
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(room).name, "Smoky Lounge");

    let unlock = |object, generic| Action::BeginRoomUnlock {
        object,
        door: RoomDoor::Right,
        cost: ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic,
        },
    };
    let room_ref = game.object_ref(room);
    let mut one = game.clone();
    one.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
    cast_named(&mut one, player, "Murder", Some(Target::Card(first)));
    settle(&mut one);
    one.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut one, unlock(room_ref, 1));
    reject(&mut one, unlock(room_ref, 2));
    one.state.players[player.index()].mana[5] = 2;
    act(&mut one, unlock(room_ref, 2));
    choose(&mut one, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    settle(&mut one);
    assert_eq!(one.state.players[player.index()].mana, [0; 6]);

    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, unlock(room_ref, 1));
    choose(&mut game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors {
            left: true,
            right: true
        })
    );
}
