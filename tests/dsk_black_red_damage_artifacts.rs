//! Authored damage, counter, life, and exact enchantment-return interactions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn announce_activation(game: &mut Game, card: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn equip(game: &mut Game, equipment: CardId, bearer: CardId) {
    announce_activation(game, equipment);
    target(game, Target::Card(bearer));
    pay(game);
    settle(game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(bearer))
    );
}

#[test]
fn chainsaw_counts_creature_death_events_and_moves_its_live_power_bonus() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let etb_target = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let saw = put(&mut game, player, "Chainsaw", Zone::Hand);
    cast(&mut game, saw, None);
    pass_pair(&mut game);
    let mut omitted = game.clone();
    act(&mut omitted, Action::FinishTargets);
    settle(&mut omitted);
    assert_eq!(omitted.object_view(etb_target).zone, Zone::Battlefield);
    assert_eq!(omitted.counter_count(saw, CounterKind::Rev), 0);

    target(&mut game, Target::Card(etb_target));
    settle(&mut game);
    assert_eq!(game.object_view(etb_target).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(saw, CounterKind::Rev), 1);
    equip(&mut game, saw, first);
    assert_eq!(stats(&game, first), (11, 10));

    let victims =
        [player, opponent].map(|owner| put(&mut game, owner, "Grizzly Bears", Zone::Battlefield));
    let sweep = put(&mut game, player, "Pyroclasm", Zone::Hand);
    cast(&mut game, sweep, None);
    settle(&mut game);
    for victim in victims {
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
    assert_eq!(game.counter_count(saw, CounterKind::Rev), 2);
    assert_eq!(stats(&game, first), (12, 10));
    equip(&mut game, saw, second);
    assert_eq!(stats(&game, first), (10, 10));
    assert_eq!(stats(&game, second), (5, 3));
}

#[test]
fn cursed_recording_clears_every_time_counter_and_uses_the_exact_departure_count() {
    let player = Player::P1;
    for (counters, leaves, expected) in [
        (5, false, 35),
        (6, false, 15),
        (8, false, 15),
        (8, true, 15),
    ] {
        let mut game = scenario(player);
        game.state.players[player.index()].life = 35;
        let source = put(&mut game, player, "Cursed Recording", Zone::Hand);
        cast(&mut game, source, None);
        settle(&mut game);
        game.add_counter(source, CounterKind::Time, counters);
        let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
        cast(&mut game, creature, None);
        settle(&mut game);
        assert_eq!(game.counter_count(source, CounterKind::Time), counters);

        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        assert_eq!(game.state.stack.len(), 2);
        if leaves {
            game.move_object(source, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, expected);
        assert_eq!(game.state.players[player.other().index()].life, 18);
        assert_eq!(
            game.counter_count(source, CounterKind::Time),
            if counters == 5 { 6 } else { 0 }
        );
    }
}

#[test]
fn cursed_recording_copies_the_next_qualifying_spell_once_after_it_leaves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Cursed Recording", Zone::Battlefield);
    announce_activation(&mut game, source);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    game.move_object(source, Zone::Graveyard);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, creature, None);
    settle(&mut game);

    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    assert!(game.retarget_view().unwrap().item.is_copy);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(blocker),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 18);

    let next = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, next, Some(Target::Player(opponent)));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
}

#[test]
fn enduring_courage_gives_haste_to_other_creatures_after_returning_as_an_enchantment() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Enduring Courage", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
    assert!(!game.object_view(source).keywords.contains(Keywords::HASTE));
    let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    game.move_object(foreign, Zone::Battlefield);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    assert_eq!(stats(&game, foreign), (2, 2));

    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    pass_pair(&mut game);
    let departed = game.object_ref(source);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let mut moved_again = game.clone();
    moved_again.move_object(source, Zone::Exile);
    settle(&mut moved_again);
    assert_eq!(moved_again.object_view(source).zone, Zone::Exile);
    settle(&mut game);
    let returned = game.object_view(source);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_ne!(returned.reference, departed);
    assert_eq!(returned.kinds, 32);
    assert!(!returned.has_subtype("Dog") && !returned.has_subtype("Glimmer"));

    let cub = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, cub, None);
    settle(&mut game);
    assert_eq!(stats(&game, cub), (4, 2));
    assert!(game.object_view(cub).keywords.contains(Keywords::HASTE));
    assert!(game.state.objects[cub as usize].summoning_sick);
    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    assert!(game.legal_actions().contains(&Action::Attack { card: cub }));
}

#[test]
fn enduring_tenacity_returns_to_its_owner_and_uses_actual_life_gained() {
    let former_controller = Player::P1;
    let owner = former_controller.other();
    let mut game = scenario(former_controller);
    let source = put(&mut game, owner, "Enduring Tenacity", Zone::Battlefield);
    game.change_controller(game.object_ref(source), former_controller);
    let removal = put(&mut game, former_controller, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    let view = game.object_view(source);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!(view.controller, owner);
    assert_eq!(view.kinds, 32);

    game.gain_life(former_controller, 2);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.gain_life(owner, 3);
    game.prepare_triggers();
    assert_eq!(game.actor(), Some(owner));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(owner),
        },
    );
    target(&mut game, Target::Player(former_controller));
    settle(&mut game);
    assert_eq!(game.state.players[former_controller.index()].life, 19);
    assert_eq!(game.state.players[owner.index()].life, 23);

    let removal = put(&mut game, former_controller, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn rollercrusher_ride_and_fear_of_burning_alive_use_live_delirium_and_actual_damage() {
    let player = Player::P1;
    let opponent = player.other();
    for delirium in [false, true] {
        let mut game = scenario(player);
        for name in ["Forest", "Jade-Cast Sentinel"] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        let enchantment = put(
            &mut game,
            player,
            "Pacifism",
            if delirium {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let own = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
        let enemy = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let ride = put(&mut game, player, "The Rollercrusher Ride", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: ride });
        act(&mut game, Action::ChooseX { value: 2 });
        pay(&mut game);
        pass_pair(&mut game);
        target(&mut game, Target::Card(own));
        target(&mut game, Target::Card(enemy));
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        let etb_damage = if delirium { 4 } else { 2 };
        assert_eq!(game.object_view(own).damage, etb_damage);
        assert_eq!(game.object_view(enemy).damage, etb_damage);

        let fear = put(&mut game, player, "Fear of Burning Alive", Zone::Hand);
        cast(&mut game, fear, None);
        pass_pair(&mut game);
        pass_pair(&mut game);
        assert_eq!(
            game.state.players[opponent.index()].life,
            if delirium { 12 } else { 16 }
        );
        if delirium {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            target(&mut game, Target::Card(enemy));
            let receipt = game.state.stack.last().unwrap().damage_event.unwrap();
            assert_eq!(receipt.player, opponent);
            assert_eq!(receipt.amount, 8);
            let mut lost_delirium = game.clone();
            lost_delirium.move_object(enchantment, Zone::Exile);
            settle(&mut lost_delirium);
            assert_eq!(lost_delirium.object_view(enemy).damage, 4);
            settle(&mut game);
            assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        } else {
            assert!(game.state.stack.is_empty());
            assert_eq!(game.object_view(enemy).damage, 2);
        }
    }
}

#[test]
fn grievous_wound_halves_post_damage_life_once_and_ends_the_live_gain_prohibition() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.players[opponent.index()].life = 23;
    let attackers =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, player, name, Zone::Battlefield));
    let wound = put(&mut game, player, "Grievous Wound", Zone::Hand);
    cast(&mut game, wound, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.object_view(wound).attached_to_player, Some(opponent));
    game.gain_life(player, 3);
    game.gain_life(opponent, 3);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 23);

    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    for card in attackers {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 9);
    game.state.players[player.index()].mana[0] = 2;
    let remove = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, remove, Some(Target::Card(wound)));
    settle(&mut game);
    assert_eq!(game.object_view(wound).zone, Zone::Graveyard);
    game.gain_life(opponent, 3);
    assert_eq!(game.state.players[opponent.index()].life, 12);
}

#[test]
fn screaming_nemesis_remains_standard_banned_and_locks_only_the_actual_player_recipient() {
    let name = "Screaming Nemesis";
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported());
    assert!(!card.is_standard_legal());
    let mut deck = vec![Card::Forest; 60];
    deck[0] = card;
    let config = Config {
        decks: [deck, vec![Card::Forest; 60]],
        seed: 20261023,
        starting_player: Player::P1,
        deck_rules: DeckRules::Standard,
    };
    assert!(
        matches!(Game::new(config.clone()), Err(Error::InvalidDeck(message)) if message.contains(name))
    );
    assert!(
        Game::new(Config {
            deck_rules: DeckRules::Freeform,
            ..config
        })
        .is_ok()
    );

    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, name, Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).keywords.contains(Keywords::HASTE));
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(source)));
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let mut permanent = game.clone();
    target(&mut permanent, Target::Card(victim));
    settle(&mut permanent);
    assert_eq!(permanent.object_view(victim).zone, Zone::Graveyard);
    permanent.gain_life(opponent, 3);
    assert_eq!(permanent.state.players[opponent.index()].life, 23);

    target(&mut game, Target::Player(opponent));
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    game.gain_life(opponent, 3);
    game.gain_life(player, 3);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].life, 23);
}
