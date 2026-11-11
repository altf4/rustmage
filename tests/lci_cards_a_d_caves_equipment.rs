//! Cave mana and quantities, modal resolution, exact Equip routes and costs.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BAT: Card = Card::from_catalogue_id(5276);
const TREASURE: Card = Card::from_catalogue_id(5302);

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..20 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario did not reach its next decision");
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn token_count(game: &Game, player: Player, card: Card) -> usize {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.is_token()
        })
        .count()
}

fn generate_cave_colorless(game: &mut Game, source: CardId) {
    choose(game, |action| {
        matches!(action,
            Action::ActivateMana { card, production: [0, 0, 0, 0, 0, 1], .. } if *card == source
        )
    });
}

#[test]
fn bat_colony_counts_cave_mana_spent_and_targets_a_controlled_creature_per_cave() {
    let player = Player::P0;
    let foe = player.other();
    for quantity in 0..=2 {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 2 - quantity];
        for _ in 0..quantity {
            let cave = put(&mut game, player, "Captivating Cave", Zone::Battlefield);
            generate_cave_colorless(&mut game, cave);
        }
        let colony = candidate(&mut game, player, "Bat Colony", Zone::Hand);
        cast(&mut game, colony, None);
        settle(&mut game);
        assert_eq!(token_count(&game, player, BAT), quantity as usize);
        for bat in game
            .observe(player)
            .objects
            .into_iter()
            .filter(|object| object.card == BAT)
        {
            assert_eq!((bat.power, bat.toughness), (1, 1));
            assert_eq!(bat.colors, 4);
            assert!(bat.keywords.contains(Keywords::FLYING));
        }
        game.move_object(colony, Zone::Exile);
        game.move_object(colony, Zone::Battlefield);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(token_count(&game, player, BAT), quantity as usize);

        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, foe, "Bear Cub", Zone::Battlefield);
        let cave = put(&mut game, player, "Captivating Cave", Zone::Hand);
        let foe_cave = put(&mut game, foe, "Captivating Cave", Zone::Hand);
        let basic = put(&mut game, player, "Forest", Zone::Hand);
        for nontrigger in [foe_cave, basic] {
            game.move_object(nontrigger, Zone::Battlefield);
            game.prepare_triggers();
            assert!(game.state.stack.is_empty());
        }
        game.move_object(cave, Zone::Battlefield);
        game.prepare_triggers();
        advance(&mut game, |game| {
            matches!(game.state.pending, Pending::Targets { .. })
        });
        reject(&mut game, target(enemy));
        act(&mut game, target(creature));
        settle(&mut game);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    }
}

#[test]
fn bloodthorn_flail_has_one_equip_ability_with_printed_and_private_discard_routes() {
    let player = Player::P1;
    let mut root = scenario(player);
    let gear = candidate(&mut root, player, "Bloodthorn Flail", Zone::Battlefield);
    let bearer = put(&mut root, player, "Bear Cub", Zone::Battlefield);
    let foe = put(
        &mut root,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let foreign = put(&mut root, player.other(), "Shock", Zone::Hand);
    let mut empty = root.clone();
    empty.state.players[player.index()].mana = [0; 6];
    reject(&mut empty, activate(gear, 0));
    let discard = put(&mut root, player, "Forest", Zone::Hand);
    act(&mut root, activate(gear, 0));
    for route in [
        ActivationCostRoute::Printed,
        ActivationCostRoute::DiscardCard,
    ] {
        let mut game = root.clone();
        let cost = game
            .observe(player)
            .cost_choices
            .into_iter()
            .find(|choice| choice.activation_route == Some(route))
            .unwrap()
            .index;
        act(&mut game, Action::ChooseActivationCost { cost });
        reject(&mut game, target(foe));
        act(&mut game, target(bearer));
        if route == ActivationCostRoute::DiscardCard {
            assert!(game.observe(player.other()).additional_cost.is_none());
            let invalid = Action::ChooseAdditionalCost {
                object: game.object_ref(foreign),
            };
            reject(&mut game, invalid);
            let payment = Action::ChooseAdditionalCost {
                object: game.object_ref(discard),
            };
            act(&mut game, payment);
            assert_eq!(game.observe(player).payment_request.unwrap().generic, 0);
        } else {
            assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, bearer), (4, 3));
        assert_eq!(
            game.object_view(gear).attached_to,
            Some(game.object_ref(bearer))
        );
        assert_eq!(
            game.object_view(discard).zone,
            if route == ActivationCostRoute::Printed {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn cave_in_damages_creatures_and_walkers_using_the_live_own_caves_in_both_zones() {
    let player = Player::P0;
    let foe = player.other();
    let mut game = scenario(player);
    let own_cave = put(&mut game, player, "Captivating Cave", Zone::Battlefield);
    let grave_cave = put(&mut game, player, "Captivating Cave", Zone::Graveyard);
    put(&mut game, foe, "Captivating Cave", Zone::Battlefield);
    put(&mut game, foe, "Captivating Cave", Zone::Graveyard);
    let ally = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, foe, "Hill Giant", Zone::Battlefield);
    let walker = put(&mut game, foe, "Chandra, Flameshaper", Zone::Battlefield);
    game.add_counter(walker, CounterKind::Loyalty, 6);
    let artifact = put(&mut game, foe, "Swiftfoot Boots", Zone::Battlefield);
    let spell = candidate(&mut game, player, "Calamitous Cave-In", Zone::Hand);
    cast(&mut game, spell, None);
    let mut no_caves = game.clone();
    no_caves.move_object(own_cave, Zone::Exile);
    no_caves.move_object(grave_cave, Zone::Exile);
    settle(&mut no_caves);
    assert_eq!(no_caves.object_view(ally).damage, 0);
    assert_eq!(no_caves.counter_count(walker, CounterKind::Loyalty), 6);
    settle(&mut game);
    assert_eq!(game.object_view(ally).damage, 2);
    assert_eq!(game.object_view(enemy).damage, 2);
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 4);
    assert_eq!(game.object_view(artifact).damage, 0);
    for owner in [player, foe] {
        assert_eq!(game.state.players[owner.index()].life, STARTING_LIFE);
    }
}

#[test]
fn cavernous_maw_qualifies_without_counting_itself_and_remains_a_cave_when_animated() {
    let player = Player::P0;
    let mut game = scenario(player);
    let maw = candidate(&mut game, player, "Cavernous Maw", Zone::Battlefield);
    let mut funded = game.clone();
    generate_cave_colorless(&mut funded, maw);
    assert_eq!(funded.state.players[player.index()].mana[5], 9);
    assert!(funded.object_view(maw).tapped);
    put(&mut game, player, "Captivating Cave", Zone::Battlefield);
    put(&mut game, player, "Captivating Cave", Zone::Graveyard);
    put(
        &mut game,
        player.other(),
        "Captivating Cave",
        Zone::Battlefield,
    );
    reject(&mut game, activate(maw, 1));
    let last = put(&mut game, player, "Captivating Cave", Zone::Graveyard);
    act(&mut game, activate(maw, 1));
    pay(&mut game);
    game.move_object(last, Zone::Exile);
    settle(&mut game);
    let land = game.object_view(maw);
    assert_eq!(land.kinds, 1 | 2);
    assert!(land.has_subtype("Elemental") && land.has_subtype("Cave"));
    assert_eq!(stats(&game, maw), (3, 3));
    reject(&mut game, activate(maw, 1));
}

#[test]
fn compass_gnome_can_decline_or_reveal_either_eligible_card_on_top() {
    let player = Player::P1;
    let mut root = scenario(player);
    let gnome = candidate(&mut root, player, "Compass Gnome", Zone::Hand);
    let basic = put(&mut root, player, "Island", Zone::Library);
    let cave = put(&mut root, player, "Captivating Cave", Zone::Library);
    let wrong = candidate(&mut root, player, "Temple Garden", Zone::Library);
    let unchanged = root.state.players[player.index()].library.clone();
    cast(&mut root, gnome, None);
    advance(&mut root, |game| {
        game.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let mut declined = root.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(declined.state.players[player.index()].library, unchanged);
    act(&mut root, Action::ChooseOptional { accept: true });
    reject(&mut root, Action::ChooseSearch { card: wrong });
    for card in [basic, cave] {
        let mut game = root.clone();
        act(&mut game, Action::ChooseSearch { card });
        act(&mut game, Action::FinishSearch);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.last().copied(),
            Some(card)
        );
        assert_eq!(stats(&game, gnome), (2, 1));
        assert_eq!(game.object_view(gnome).kinds, 17);
    }
}

fn confluence(game: &mut Game, spell: CardId, mode: u16, targets: &[CardId]) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    for &card in targets {
        act(game, target(card));
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn choose_cave(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseEffectObject { object });
}

#[test]
fn cosmium_confluence_uses_printed_mode_order_and_independent_repeated_targets() {
    let player = Player::P0;
    let foe = player.other();
    let mut root = scenario(player);
    let own = put(&mut root, player, "Captivating Cave", Zone::Battlefield);
    let foreign = put(&mut root, foe, "Captivating Cave", Zone::Battlefield);
    let found = put(&mut root, player, "Captivating Cave", Zone::Library);
    let victim = put(&mut root, foe, "Impact Tremors", Zone::Battlefield);
    let second = put(&mut root, foe, "Rest in Peace", Zone::Battlefield);
    let wrong = put(&mut root, foe, "Swiftfoot Boots", Zone::Battlefield);
    let spell = candidate(&mut root, player, "Cosmium Confluence", Zone::Hand);

    let mut mixed = root.clone();
    confluence(&mut mixed, spell, 4, &[victim]);
    advance(&mut mixed, |game| {
        matches!(game.state.pending, Pending::EffectSearch { .. })
    });
    assert_eq!(mixed.object_view(victim).zone, Zone::Battlefield);
    act(&mut mixed, Action::ChooseSearch { card: found });
    act(&mut mixed, Action::FinishSearch);
    let invalid = Action::ChooseEffectObject {
        object: mixed.object_ref(foreign),
    };
    reject(&mut mixed, invalid);
    choose_cave(&mut mixed, found);
    settle(&mut mixed);
    let cave = mixed.object_view(found);
    assert_eq!(cave.kinds, 3);
    assert!(cave.tapped && cave.has_subtype("Cave") && cave.has_subtype("Elemental"));
    assert!(cave.keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&mixed, found), (3, 3));
    assert_eq!(mixed.object_view(victim).zone, Zone::Exile);

    let mut animated = root.clone();
    confluence(&mut animated, spell, 6, &[]);
    advance(&mut animated, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    for _ in 0..3 {
        choose_cave(&mut animated, own);
    }
    settle(&mut animated);
    assert_eq!(stats(&animated, own), (9, 9));
    assert_eq!(animated.counter_count(own, CounterKind::PlusOne), 9);

    act(&mut root, Action::AnnounceCast { card: spell });
    act(&mut root, Action::ChooseMode { mode: 9 });
    reject(&mut root, target(wrong));
    for card in [victim, victim, second] {
        act(&mut root, target(card));
    }
    if root.legal_actions().contains(&Action::FinishTargets) {
        act(&mut root, Action::FinishTargets);
    }
    pay(&mut root);
    let mut shielded = root.clone();
    shielded.add_counter(victim, CounterKind::Shield, 1);
    settle(&mut shielded);
    assert_eq!(shielded.object_view(victim).zone, Zone::Exile);
    assert_eq!(shielded.object_view(second).zone, Zone::Exile);
    root.move_object(victim, Zone::Exile);
    settle(&mut root);
    assert_eq!(root.object_view(second).zone, Zone::Exile);
}

#[test]
fn dauntless_dismantler_taps_opposing_artifacts_and_uses_both_x_symbols() {
    let player = Player::P0;
    let foe = player.other();
    let mut root = scenario(player);
    let source = candidate(&mut root, player, "Dauntless Dismantler", Zone::Battlefield);
    let own = put(&mut root, player, "Swiftfoot Boots", Zone::Hand);
    let theirs = put(&mut root, foe, "Swiftfoot Boots", Zone::Hand);
    root.move_object(own, Zone::Battlefield);
    root.move_object(theirs, Zone::Battlefield);
    assert!(!root.object_view(own).tapped);
    assert!(root.object_view(theirs).tapped);
    let token = spawn(&mut root, foe, TREASURE, Zone::Battlefield);
    let one = put(&mut root, player, "Soul-Guide Lantern", Zone::Battlefield);
    let nonartifact = put(&mut root, foe, "Bear Cub", Zone::Battlefield);
    for value in [0, 2] {
        let mut game = root.clone();
        act(&mut game, activate(source, 0));
        act(&mut game, Action::ChooseX { value });
        let request = game.observe(player).payment_request.unwrap();
        assert_eq!(request.generic, 2 * value);
        assert_eq!(request.required, [1, 0, 0, 0, 0, 0]);
        pay(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        settle(&mut game);
        for card in [own, theirs] {
            assert_eq!(
                game.object_view(card).zone,
                if value == 2 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
        assert_eq!(
            game.object_view(token).zone == Zone::Battlefield,
            value != 0
        );
        assert_eq!(game.object_view(one).zone, Zone::Battlefield);
        assert_eq!(game.object_view(nonartifact).zone, Zone::Battlefield);
    }
}

#[test]
fn deconstruction_hammer_sacrifices_only_the_selected_grantor() {
    let player = Player::P1;
    let foe = player.other();
    let mut game = scenario(player);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hammers = [
        candidate(
            &mut game,
            player,
            "Deconstruction Hammer",
            Zone::Battlefield,
        ),
        candidate(
            &mut game,
            player,
            "Deconstruction Hammer",
            Zone::Battlefield,
        ),
    ];
    let foreign = candidate(&mut game, foe, "Deconstruction Hammer", Zone::Battlefield);
    let victim = put(&mut game, foe, "Impact Tremors", Zone::Battlefield);
    for gear in hammers {
        act(&mut game, activate(gear, 0));
        act(&mut game, target(bearer));
        assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(stats(&game, bearer), (4, 4));
    let bearer_ref = game.object_ref(bearer);
    let foreign_ref = game.object_ref(foreign);
    assert!(game.attach_equipment(foreign_ref, bearer_ref));
    reject(
        &mut game,
        Action::AnnounceAttachedAbility {
            grantor: foreign_ref,
            ability: 1,
        },
    );
    let grantor = game.object_ref(hammers[1]);
    let use_hammer = Action::AnnounceAttachedAbility {
        grantor,
        ability: 1,
    };
    game.state.objects[bearer as usize].summoning_sick = true;
    reject(&mut game, use_hammer.clone());
    game.state.objects[bearer as usize].summoning_sick = false;
    act(&mut game, use_hammer);
    reject(&mut game, target(bearer));
    act(&mut game, target(victim));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);
    pay(&mut game);
    assert_eq!(game.object_view(hammers[1]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(hammers[0]).attached_to, Some(bearer_ref));
    assert_eq!(game.object_view(foreign).attached_to, Some(bearer_ref));
    assert!(game.object_view(bearer).tapped);
    game.move_object(bearer, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}
