//! Authored black and red Rooms, their independent doors, and their exact effects.
mod support;

use rustmage::RoomDoor;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLACK: usize = 2;
const RED: usize = 3;

fn mana(color: usize, colored: u32, generic: u32) -> ManaCost {
    let mut cost = ManaCost {
        generic,
        ..ManaCost::ZERO
    };
    cost.colored[color] = colored;
    cost
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn cast_room(game: &mut Game, player: Player, name: &str, door: RoomDoor) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    act(
        game,
        Action::AnnounceRoom {
            card,
            door,
            flash_surcharge: false,
        },
    );
    pay(game);
    pass_pair(game);
    card
}

fn unlock(game: &mut Game, card: CardId, door: RoomDoor, cost: ManaCost) {
    let object = game.object_ref(card);
    act(game, Action::BeginRoomUnlock { object, door, cost });
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn step(game: &mut Game, next: Step) {
    game.begin_step(next);
    game.prepare_triggers();
}

fn settle_room(game: &mut Game) {
    for _ in 0..32 {
        match game.state.pending {
            Pending::Priority if game.state.stack.is_empty() && game.state.resolving.is_none() => {
                return;
            }
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            ref pending => panic!("unexpected Room continuation {pending:?}"),
        }
    }
    panic!("Room resolution did not finish");
}

fn declare_attacks(game: &mut Game, attackers: &[CardId]) {
    step(game, Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn charred_foyer_gives_permission_and_warped_space_supplies_one_zero_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let top = put(&mut game, player, "Quick Study", Zone::Library);
    let other = put(&mut game, player, "Quick Study", Zone::Exile);
    let room = cast_room(
        &mut game,
        player,
        "Charred Foyer // Warped Space",
        RoomDoor::Left,
    );
    settle_room(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Library);
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors {
            left: true,
            right: false
        })
    );

    step(&mut game, Step::Upkeep);
    settle_room(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    step(&mut game, Step::Main1);
    let provider = game.object_ref(room);
    let zero = |game: &Game, card| Action::AnnounceExileForZero {
        object: game.object_ref(card),
        provider,
        face: CastFace::Normal,
        flash_surcharge: false,
        origin: ExileZeroOrigin::Ordinary,
    };
    let locked = zero(&game, top);
    reject(&mut game, locked);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card: top })
    );

    unlock(&mut game, room, RoomDoor::Right, mana(RED, 2, 4));
    let unavailable = zero(&game, other);
    reject(&mut game, unavailable);
    let cast = zero(&game, top);
    act(&mut game, cast);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0; 6]
    );
    settle_room(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);

    game.grant_exile_play_permission(player, game.object_ref(other), false);
    let used = zero(&game, other);
    reject(&mut game, used);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card: other })
    );
}

#[test]
fn defiled_crypt_makes_one_horror_when_cadaver_lab_returns_a_creature() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let returnee = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let second = put(&mut game, player, "Opt", Zone::Graveyard);
    let other = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let room = cast_room(
        &mut game,
        player,
        "Defiled Crypt // Cadaver Lab",
        RoomDoor::Left,
    );
    settle_room(&mut game);
    unlock(&mut game, room, RoomDoor::Right, mana(BLACK, 1, 0));
    reject(&mut game, target(Target::Card(other)));
    reject(&mut game, target(Target::Card(second)));
    act(&mut game, target(Target::Card(returnee)));
    settle_room(&mut game);
    assert_eq!(game.object_view(returnee).zone, Zone::Hand);

    let horror = game.controlled_token_ids(player).next().unwrap();
    let token = game.object_view(horror);
    assert_eq!(stats(&game, horror), (2, 2));
    assert_eq!(token.kinds, 1 | 32);
    assert_eq!(token.colors, 4);
    assert!(token.has_subtype("Horror"));
    game.move_object(second, Zone::Hand);
    game.prepare_triggers();
    settle_room(&mut game);
    assert_eq!(game.controlled_token_ids(player).count(), 1);
}

#[test]
fn derelict_attic_draws_and_widows_walk_grants_only_the_lone_attacker_deathtouch() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let bystander = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let room = cast_room(
        &mut game,
        player,
        "Derelict Attic // Widow's Walk",
        RoomDoor::Left,
    );
    settle_room(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.state.players[player.index()].life, 18);

    let mut locked = game.clone();
    unlock(&mut game, room, RoomDoor::Right, mana(BLACK, 1, 3));
    for (branch, power) in [(&mut locked, 2), (&mut game, 3)] {
        declare_attacks(branch, &[attacker]);
        settle_room(branch);
        assert_eq!(stats(branch, attacker), (power, 2));
        assert_eq!(
            branch
                .object_view(attacker)
                .keywords
                .contains(Keywords::DEATHTOUCH),
            power == 3,
        );
        assert_eq!(stats(branch, bystander), (2, 2));
    }
}

#[test]
fn funeral_room_drains_per_death_and_awakening_hall_returns_own_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let own =
        ["Bear Cub", "Savannah Lions"].map(|name| put(&mut game, player, name, Zone::Battlefield));
    let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Forest", Zone::Graveyard);
    let room = cast_room(
        &mut game,
        player,
        "Funeral Room // Awakening Hall",
        RoomDoor::Left,
    );
    let pyroclasm = put(&mut game, player, "Pyroclasm", Zone::Hand);
    cast(&mut game, pyroclasm, None);
    settle_room(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert!(
        own.iter()
            .all(|&card| game.object_view(card).zone == Zone::Graveyard)
    );
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);

    unlock(&mut game, room, RoomDoor::Right, mana(BLACK, 2, 6));
    settle_room(&mut game);
    assert!(
        own.iter()
            .all(|&card| game.object_view(card).zone == Zone::Battlefield)
    );
    for card in [foreign, noncreature] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn glassworks_targets_opposing_creatures_and_shattered_yard_damages_at_end() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let room = cast_room(
        &mut game,
        player,
        "Glassworks // Shattered Yard",
        RoomDoor::Right,
    );
    settle_room(&mut game);
    assert_eq!(game.object_view(foreign).damage, 0);
    unlock(&mut game, room, RoomDoor::Left, mana(RED, 1, 2));
    reject(&mut game, target(Target::Card(own)));
    act(&mut game, target(Target::Card(foreign)));
    settle_room(&mut game);
    assert_eq!(game.object_view(foreign).damage, 4);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    step(&mut game, Step::End);
    settle_room(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}

#[test]
fn painters_studio_allows_both_exile_plays_and_defaced_gallery_grows_attackers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacking =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, player, name, Zone::Battlefield));
    let bystander = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Quick Study", Zone::Library);
    let room = cast_room(
        &mut game,
        player,
        "Painter's Studio // Defaced Gallery",
        RoomDoor::Left,
    );
    settle_room(&mut game);
    for card in [land, spell] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card: spell })
    );
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);

    unlock(&mut game, room, RoomDoor::Right, mana(RED, 1, 1));
    declare_attacks(&mut game, &attacking);
    settle_room(&mut game);
    for card in attacking {
        assert_eq!(stats(&game, card), (3, 2));
    }
    assert_eq!(stats(&game, bystander), (3, 3));
}

#[test]
fn ticket_booth_manifests_and_tunnel_of_hate_grants_double_strike() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let grave = put(&mut game, player, "Forest", Zone::Library);
    let face_down = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let room = cast_room(
        &mut game,
        player,
        "Ticket Booth // Tunnel of Hate",
        RoomDoor::Left,
    );
    pass_pair(&mut game);
    assert!(game.observe(opponent).manifest_choice.is_none());
    let object = game.object_ref(face_down);
    let choice = Action::ChooseManifest { object };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| action == &choice)
        .unwrap();
    let pending_manifest = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, choice);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&pending_manifest).unwrap();
    assert_eq!(indexed.snapshot(), pending_manifest);
    assert!(game.is_face_down(game.object_ref(face_down)));
    assert_eq!(game.object_view(grave).zone, Zone::Graveyard);
    assert_eq!(
        game.observe(opponent).face_down_permanents[0].underlying,
        None
    );

    unlock(&mut game, room, RoomDoor::Right, mana(RED, 2, 4));
    declare_attacks(&mut game, &[attacker]);
    reject(&mut game, target(Target::Card(face_down)));
    act(&mut game, target(Target::Card(attacker)));
    settle_room(&mut game);
    assert!(
        game.object_view(attacker)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
}

#[test]
fn unholy_annex_draws_then_checks_the_demon_from_ritual_chamber() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let room = cast_room(
        &mut game,
        player,
        "Unholy Annex // Ritual Chamber",
        RoomDoor::Left,
    );
    let mut ritual = game.clone();
    unlock(&mut ritual, room, RoomDoor::Right, mana(BLACK, 2, 3));
    settle_room(&mut ritual);
    let demon = ritual.controlled_token_ids(player).next().unwrap();
    let token = ritual.object_view(demon);
    assert_eq!(stats(&ritual, demon), (6, 6));
    assert_eq!(token.kinds, 1);
    assert_eq!(token.colors, 4);
    assert!(token.has_subtype("Demon"));
    assert!(token.keywords.contains(Keywords::FLYING));

    for (branch, own_life, other_life) in [(&mut game, 18, 20), (&mut ritual, 22, 18)] {
        step(branch, Step::End);
        settle_room(branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            1
        );
        assert_eq!(branch.state.players[player.index()].life, own_life);
        assert_eq!(branch.state.players[opponent.index()].life, other_life);
    }
}
