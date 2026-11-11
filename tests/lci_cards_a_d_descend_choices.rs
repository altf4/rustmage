//! Descend and historical entry conditions, plus the set's small choice effects.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FUNGUS: Card = Card::from_catalogue_id(5292);
const TREASURE: Card = Card::from_catalogue_id(5302);
const SKELETON_PIRATE: Card = Card::from_catalogue_id(5577);
const VAMPIRE_DEMON: Card = Card::from_catalogue_id(5578);

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter(game: &mut Game, card: CardId) {
    cast(game, card, None);
    pass_pair(game);
}

fn end(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

fn token(game: &Game, player: Player, card: Card) -> ObjectView {
    game.observe(player)
        .objects
        .into_iter()
        .find(|o| {
            o.zone == Zone::Battlefield && o.controller == player && o.card == card && o.is_token()
        })
        .expect("expected token")
}

fn effect_target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn permanent_arrival_history_powers_the_three_end_step_creatures() {
    for (name, keyword) in [
        ("Child of the Volcano", Keywords::TRAMPLE),
        ("Deep Goblin Skulltaker", Keywords::MENACE),
        ("Broodrage Mycoid", Keywords::NONE),
    ] {
        let player = Player::P0;
        let mut game = scenario(player);
        let source = candidate(&mut game, player, name, Zone::Battlefield);
        let permanent = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        let instant = put(&mut game, player, "Shock", Zone::Hand);
        assert!(game.object_view(source).keywords.contains(keyword));
        let mut absent = game.clone();
        absent.move_object(instant, Zone::Graveyard);
        end(&mut absent);
        assert!(absent.state.stack.is_empty(), "{name}");
        game.move_object(permanent, Zone::Graveyard);
        game.move_object(permanent, Zone::Exile);
        end(&mut game);
        assert_eq!(game.state.stack.len(), 1, "{name}");
        settle(&mut game);
        if name == "Broodrage Mycoid" {
            let created = token(&game, player, FUNGUS);
            assert!(created.has_subtype("Fungus"));
            assert_eq!(created.colors, 4);
            assert!(created.keywords.contains(Keywords::CANT_BLOCK));
            assert_eq!(stats(&game, created.id), (1, 1));
        } else {
            assert_eq!(
                game.counter_count(source, CounterKind::PlusOne),
                1,
                "{name}"
            );
        }
    }
}

#[test]
fn canonized_in_blood_targets_a_controlled_creature_and_sacrifices_for_the_demon() {
    let player = Player::P1;
    let mut game = scenario(player);
    let enchantment = candidate(&mut game, player, "Canonized in Blood", Zone::Battlefield);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let deposited = put(&mut game, player, "Forest", Zone::Hand);
    let mut quiet = game.clone();
    end(&mut quiet);
    assert!(quiet.state.stack.is_empty());
    game.move_object(deposited, Zone::Graveyard);
    end(&mut game);
    reject(&mut game, effect_target(enemy));
    act(&mut game, effect_target(own));
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);

    let activation = Action::AnnounceActivate {
        card: enchantment,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut short = game.clone();
    short.state.players[player.index()].mana = [0, 0, 1, 0, 0, 6];
    reject(&mut short, activation.clone());
    act(&mut game, activation);
    pay(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    settle(&mut game);
    let demon = token(&game, player, VAMPIRE_DEMON);
    assert_eq!(stats(&game, demon.id), (4, 3));
    assert_eq!(demon.colors, 1 | 4);
    assert!(demon.has_subtype("Vampire") && demon.has_subtype("Demon"));
    assert!(demon.keywords.contains(Keywords::FLYING));
}

#[test]
fn corpses_of_the_lost_grants_a_live_skeleton_anthem_and_offers_the_paid_return() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = candidate(&mut game, player, "Corpses of the Lost", Zone::Hand);
    let permanent = put(&mut game, player, "Forest", Zone::Hand);
    enter(&mut game, source);
    settle(&mut game);
    let skeleton = token(&game, player, SKELETON_PIRATE);
    assert_eq!(stats(&game, skeleton.id), (3, 2));
    assert_eq!(skeleton.colors, 4);
    assert!(skeleton.has_subtype("Skeleton") && skeleton.has_subtype("Pirate"));
    assert!(skeleton.keywords.contains(Keywords::HASTE));
    let mut no_history = game.clone();
    end(&mut no_history);
    assert!(no_history.state.stack.is_empty());

    game.move_object(permanent, Zone::Graveyard);
    end(&mut game);
    pass_pair(&mut game);
    let saved = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 20);
    game.restore(&saved).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(source).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(stats(&game, skeleton.id), (2, 2));
    assert!(
        !game
            .object_view(skeleton.id)
            .keywords
            .contains(Keywords::HASTE)
    );
}

#[test]
fn akal_pakal_looks_privately_after_an_artifact_entered_on_the_opponents_turn() {
    let active = Player::P0;
    let player = active.other();
    let mut game = scenario(active);
    let put_in_hand = put(&mut game, player, "Opt", Zone::Library);
    let put_in_graveyard = put(&mut game, player, "Forest", Zone::Library);
    let mut wrong_player = game.clone();
    wrong_player.create_token_batch_capture(active, TREASURE, 1, false, None, true);
    candidate(
        &mut wrong_player,
        player,
        "Akal Pakal, First Among Equals",
        Zone::Battlefield,
    );
    end(&mut wrong_player);
    assert!(wrong_player.state.stack.is_empty());

    let artifact = game.create_token_batch_capture(player, TREASURE, 1, false, None, true)[0];
    game.move_object(artifact.card, Zone::Exile);
    candidate(
        &mut game,
        player,
        "Akal Pakal, First Among Equals",
        Zone::Battlefield,
    );
    end(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(active).library_selection.is_none());
    reject(&mut game, Action::FinishLibrarySelection);
    act(&mut game, Action::SelectLibrary { card: put_in_hand });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert_eq!(game.object_view(put_in_hand).zone, Zone::Hand);
    assert_eq!(game.object_view(put_in_graveyard).zone, Zone::Graveyard);
}

#[test]
fn another_chance_returns_a_chosen_subset_with_or_without_the_optional_mill() {
    let player = Player::P1;
    let mut game = scenario(player);
    let old = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let waiting = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let new = put(&mut game, player, "Bear Cub", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let enemy = put(&mut game, player.other(), "Savannah Lions", Zone::Graveyard);
    let spell = candidate(&mut game, player, "Another Chance", Zone::Hand);
    enter(&mut game, spell);
    let mut zero = game.clone();
    act(&mut zero, Action::ChooseOptional { accept: false });
    act(&mut zero, Action::FinishGraveyardReturn);
    assert_eq!(zero.object_view(old).zone, Zone::Graveyard);
    assert_eq!(zero.object_view(waiting).zone, Zone::Graveyard);
    let mut no_mill = game.clone();
    act(&mut no_mill, Action::ChooseOptional { accept: false });
    let chosen = Action::SelectGraveyardReturn {
        object: no_mill.object_ref(old),
    };
    act(&mut no_mill, chosen);
    act(&mut no_mill, Action::FinishGraveyardReturn);
    assert_eq!(no_mill.object_view(new).zone, Zone::Library);
    assert_eq!(no_mill.object_view(old).zone, Zone::Hand);
    assert_eq!(no_mill.object_view(waiting).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseOptional { accept: true });
    for card in [land, enemy] {
        let action = Action::SelectGraveyardReturn {
            object: game.object_ref(card),
        };
        reject(&mut game, action);
    }
    for card in [old, new] {
        let action = Action::SelectGraveyardReturn {
            object: game.object_ref(card),
        };
        act(&mut game, action);
    }
    assert_eq!(game.object_view(old).zone, Zone::Hand);
    assert_eq!(game.object_view(new).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
}

#[test]
fn armored_kincaller_counts_revelation_or_another_dinosaur_once() {
    let player = Player::P0;
    for another in [false, true] {
        let mut game = scenario(player);
        let source = candidate(&mut game, player, "Armored Kincaller", Zone::Hand);
        let dinosaur = put(&mut game, player, "Colossadactyl", Zone::Hand);
        let invalid = put(&mut game, player, "Hill Giant", Zone::Hand);
        if another {
            put(&mut game, player, "Colossadactyl", Zone::Battlefield);
        }
        enter(&mut game, source);
        pass_pair(&mut game);
        assert!(game.observe(player).hand_choice.is_some());
        assert!(game.observe(player.other()).hand_choice.is_none());
        let invalid = Action::ChooseHandCard {
            object: Some(game.object_ref(invalid)),
        };
        reject(&mut game, invalid);
        let mut declined = game.clone();
        act(&mut declined, Action::ChooseHandCard { object: None });
        assert_eq!(
            declined.state.players[player.index()].life,
            if another { 23 } else { 20 }
        );
        let object = game.object_ref(dinosaur);
        act(
            &mut game,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
        assert_eq!(game.state.players[player.index()].life, 23);
        assert!(game.observe(player.other()).known_hand.contains(&object));
    }
}

#[test]
fn chupacabra_echo_counts_live_permanents_and_council_rechecks_descend_four() {
    let player = Player::P1;
    let mut game = scenario(player);
    let opponent = player.other();
    let one = put(&mut game, player, "Forest", Zone::Graveyard);
    let two = put(&mut game, player, "Candy Trail", Zone::Graveyard);
    put(&mut game, player, "Opt", Zone::Graveyard);
    let ours = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let echo = candidate(&mut game, player, "Chupacabra Echo", Zone::Hand);
    enter(&mut game, echo);
    reject(&mut game, effect_target(ours));
    act(&mut game, effect_target(theirs));
    game.move_object(two, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, theirs), (2, 2));
    assert_eq!(stats(&game, ours), (3, 3));

    for name in ["Island", "Short Bow", "Grizzly Bears"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let council = candidate(&mut game, player, "Council of Echoes", Zone::Hand);
    enter(&mut game, council);
    assert!(
        game.object_view(council)
            .keywords
            .contains(Keywords::FLYING)
    );
    reject(&mut game, effect_target(land));
    reject(&mut game, effect_target(council));
    let mut none = game.clone();
    act(&mut none, Action::FinishTargets);
    settle(&mut none);
    assert_eq!(none.object_view(theirs).zone, Zone::Battlefield);
    act(&mut game, effect_target(theirs));
    let mut dropped = game.clone();
    dropped.move_object(one, Zone::Exile);
    settle(&mut dropped);
    assert_eq!(dropped.object_view(theirs).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(theirs).zone, Zone::Hand);
}
