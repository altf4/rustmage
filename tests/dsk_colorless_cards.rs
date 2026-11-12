//! Paid rules scenarios for the physical Duskmourn artifacts.
mod support;

use rustmage::{RoomChange, RoomDoor};
use rustmage_core::{cards::*, engine::*};
use support::*;

const P: Player = Player::P0;

fn ability(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(id: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(id),
    }
}

fn choose_target(game: &mut Game, id: CardId) {
    act(game, target(id));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, card: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
    pass_pair(game);
}

#[test]
fn attack_in_the_box_can_trade_its_exact_permanent_for_four_power() {
    let mut game = scenario(P);
    let attacker = put(&mut game, P, "Attack-in-the-Box", Zone::Battlefield);
    let exact = game.object_ref(attacker);
    attack(&mut game, attacker);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(stats(&decline, attacker), (2, 4));
    assert_eq!(stats(&game, attacker), (6, 4));

    let mut blink = game.clone();
    blink.move_object(attacker, Zone::Exile);
    blink.move_object(attacker, Zone::Battlefield);
    assert_ne!(blink.object_ref(attacker), exact);
    for (branch, expected) in [
        (&mut game, Zone::Graveyard),
        (&mut decline, Zone::Battlefield),
        (&mut blink, Zone::Battlefield),
    ] {
        branch.begin_step(Step::End);
        branch.prepare_triggers();
        settle(branch);
        assert_eq!(branch.object_view(attacker).zone, expected);
    }
}

#[test]
fn the_manifesting_equipment_attaches_and_each_equip_route_pays_its_own_cost() {
    for (name, toughness, keywords) in [
        ("Conductive Machete", 3, Keywords::NONE),
        (
            "Dissection Tools",
            4,
            Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
        ),
    ] {
        let mut game = scenario(P);
        let equipment = put(&mut game, P, name, Zone::Hand);
        let choice = put(&mut game, P, "Grizzly Bears", Zone::Library);
        let bearer = put(&mut game, P, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, P.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut game, equipment, None);
        pass_pair(&mut game);
        pass_pair(&mut game);
        let original = game.object_ref(choice);
        act(&mut game, Action::ChooseManifest { object: original });
        settle(&mut game);
        let manifested = game.object_ref(choice);
        assert!(game.is_face_down(manifested));
        assert_eq!(game.object_view(equipment).attached_to, Some(manifested));
        assert_eq!(stats(&game, choice), (4, toughness));
        assert!(game.keywords(choice).contains(keywords));

        let mut instant = game.clone();
        instant.state.step = Step::BeginCombat;
        reject(&mut instant, ability(equipment, 0));
        act(&mut game, ability(equipment, 0));
        reject(&mut game, target(opposing));
        choose_target(&mut game, bearer);
        if name == "Dissection Tools" {
            let mut paid_bearer = game.clone();
            let reference = paid_bearer.object_ref(bearer);
            act(
                &mut paid_bearer,
                Action::ChooseAdditionalCost { object: reference },
            );
            pay(&mut paid_bearer);
            settle(&mut paid_bearer);
            assert_eq!(paid_bearer.object_view(bearer).zone, Zone::Graveyard);
            assert_eq!(
                paid_bearer.object_view(equipment).attached_to,
                Some(manifested)
            );
            act(
                &mut game,
                Action::ChooseAdditionalCost { object: manifested },
            );
        } else {
            let unfunded = Action::PayCosts { payment: [0; 6] };
            reject(&mut game, unfunded);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(
            game.object_view(equipment).attached_to,
            Some(game.object_ref(bearer))
        );
        assert_eq!(stats(&game, bearer), (4, toughness));
        assert!(game.keywords(bearer).contains(keywords));
    }
}

#[test]
fn found_footage_can_see_an_opponents_face_down_creature_and_surveils_before_drawing() {
    let viewer = P;
    let owner = P.other();
    let mut game = scenario(owner);
    let footage = put(&mut game, viewer, "Found Footage", Zone::Battlefield);
    let smuggler = spawn(
        &mut game,
        owner,
        Card::from_name("Bubble Smuggler").unwrap(),
        Zone::Hand,
    );
    let mut no_footage = game.clone();
    no_footage.move_object(footage, Zone::Graveyard);
    for branch in [&mut game, &mut no_footage] {
        act(branch, Action::AnnounceDisguise { card: smuggler });
        pay(branch);
        settle(branch);
    }
    assert_eq!(
        no_footage.observe(viewer).face_down_permanents[0].underlying,
        None
    );
    assert_eq!(
        game.observe(viewer).face_down_permanents[0].underlying,
        Card::from_name("Bubble Smuggler"),
    );

    let take = put(&mut game, viewer, "Island", Zone::Library);
    let bin = put(&mut game, viewer, "Shock", Zone::Library);
    act(&mut game, Action::Pass);
    act(&mut game, ability(footage, 0));
    pay(&mut game);
    assert_eq!(game.object_view(footage).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let discard = Action::ArrangeLibrary {
        card: bin,
        destination: LibraryDestination::Graveyard,
    };
    assert!(!game.observe(owner).actions.contains(&discard));
    act(&mut game, discard);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: take,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(bin).zone, Zone::Graveyard);
    assert_eq!(game.object_view(take).zone, Zone::Hand);
    assert_eq!(game.state.players[viewer.index()].cards_drawn_this_turn, 1);
}

#[test]
fn haunted_screen_offers_both_mana_abilities_and_animates_once_per_incarnation() {
    let mut game = scenario(P);
    let screen = put(&mut game, P, "Haunted Screen", Zone::Battlefield);
    for (ability_index, color, life) in [(0, 0, 20), (0, 2, 20), (1, 1, 19), (1, 3, 19), (1, 4, 19)]
    {
        let mut mana_game = game.clone();
        mana_game.state.players[P.index()].mana = [0; 6];
        let mut production = [0; 6];
        production[color] = 1;
        act(
            &mut mana_game,
            Action::ActivateMana {
                card: screen,
                ability: ability_index,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            },
        );
        assert_eq!(mana_game.state.players[P.index()].mana, production);
        assert_eq!(mana_game.state.players[P.index()].life, life);
        assert!(mana_game.object_view(screen).tapped);
    }
    act(&mut game, ability(screen, 2));
    pay(&mut game);
    settle(&mut game);
    let animated = game.object_view(screen);
    assert_eq!(animated.kinds & 17, 17);
    assert!(animated.has_subtype("Spirit"));
    assert_eq!((animated.power, animated.toughness), (7, 7));
    assert_eq!(game.counter_count(screen, CounterKind::PlusOne), 7);
    reject(&mut game, ability(screen, 2));

    game.move_object(screen, Zone::Exile);
    game.move_object(screen, Zone::Battlefield);
    assert!(!game.object_view(screen).is_creature());
    assert!(game.legal_actions().contains(&ability(screen, 2)));
}

#[test]
fn keys_search_for_a_basic_and_toggle_only_a_controlled_rooms_door() {
    let mut game = scenario(P);
    let keys = put(&mut game, P, "Keys to the House", Zone::Battlefield);
    let another = put(&mut game, P, "Keys to the House", Zone::Battlefield);
    let nonbasic = put(&mut game, P, "Abandoned Campground", Zone::Library);
    let basic = put(&mut game, P, "Island", Zone::Library);
    let room_name = "Bottomless Pool // Locker Room";
    let room = put(&mut game, P, room_name, Zone::Battlefield);
    let enemy = put(&mut game, P.other(), room_name, Zone::Battlefield);
    act(&mut game, ability(keys, 0));
    pay(&mut game);
    assert_eq!(game.object_view(keys).zone, Zone::Graveyard);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    let mut fail_to_find = game.clone();
    act(&mut fail_to_find, Action::FinishSearch);
    assert_eq!(fail_to_find.object_view(basic).zone, Zone::Library);
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    assert!(
        game.observe(P.other())
            .known_hand
            .contains(&game.object_ref(basic))
    );

    let mut combat = game.clone();
    combat.state.step = Step::BeginCombat;
    reject(&mut combat, ability(another, 1));
    act(&mut game, ability(another, 1));
    reject(&mut game, target(enemy));
    choose_target(&mut game, room);
    pay(&mut game);
    assert_eq!(game.object_view(another).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let object = game.object_ref(room);
    act(
        &mut game,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Right,
            change: RoomChange::Unlock,
        },
    );
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors {
            left: false,
            right: true
        })
    );
    assert_eq!(
        game.object_view(enemy).room_doors,
        Some(RoomDoors::default())
    );
}

#[test]
fn saw_excludes_itself_and_the_equipped_attacker_and_can_sacrifice_a_land() {
    let mut game = scenario(P);
    let saw = put(&mut game, P, "Saw", Zone::Battlefield);
    let bearer = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, P, "Forest", Zone::Battlefield);
    act(&mut game, ability(saw, 0));
    choose_target(&mut game, bearer);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (4, 2));
    attack(&mut game, bearer);
    for excluded in [saw, bearer] {
        let object = game.object_ref(excluded);
        reject(&mut game, Action::Sacrifice { object });
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.state.players[P.index()].cards_drawn_this_turn, 0);
    let object = game.object_ref(land);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[P.index()].cards_drawn_this_turn, 1);
}

#[test]
fn marvin_pays_an_inherited_mana_ability_with_its_own_tap() {
    let mut game = scenario(P);
    let marvin = put(&mut game, P, "Marvin, Murderous Mimic", Zone::Battlefield);
    let donor = put(&mut game, P, "Llanowar Elves", Zone::Battlefield);
    let opposing = put(&mut game, P.other(), "Llanowar Elves", Zone::Battlefield);
    put(&mut game, P, "Forest", Zone::Battlefield);
    let source = game.object_ref(marvin);
    let rows = game
        .observe(P)
        .inherited_abilities
        .into_iter()
        .filter(|row| row.source == source)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].donor, game.object_ref(donor));
    assert_eq!(stats(&game, marvin), (2, 2));

    let production = [0, 0, 0, 0, 1, 0];
    let action = Action::ActivateInheritedMana {
        source,
        slot: rows[0].slot,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    game.state.players[P.index()].mana = [0; 6];
    let mut removed = game.clone();
    removed.move_object(donor, Zone::Exile);
    reject(&mut removed, action.clone());
    act(&mut game, action);
    assert_eq!(game.state.players[P.index()].mana, production);
    assert!(game.object_view(marvin).tapped);
    assert!(!game.object_view(donor).tapped);
    assert!(!game.object_view(opposing).tapped);
}
