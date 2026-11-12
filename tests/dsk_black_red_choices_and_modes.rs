//! Authored payment choices, spell modes, target changes, and temporary control.
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

fn cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn sacrifice(game: &Game, card: CardId) -> Action {
    Action::Sacrifice {
        object: game.object_ref(card),
    }
}

fn mode(game: &mut Game, card: CardId, mode: u16) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
}

#[test]
fn boilerbilges_ripper_selects_a_target_only_after_the_optional_sacrifice() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Boilerbilges Ripper", Zone::Hand);
    let payable = put(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    for card in [source, artifact] {
        let action = cost_object(&game, card);
        reject(&mut game, action);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(payable).zone, Zone::Battlefield);
    assert_eq!(decline.state.players[opponent.index()].life, 20);

    let choose_cost = cost_object(&game, payable);
    act(&mut game, choose_cost);
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(payable).zone, Zone::Graveyard);
    target(&mut game, Target::Player(opponent));
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn lets_play_a_game_allows_every_nonempty_subset_only_when_cast_with_delirium() {
    let player = Player::P0;
    let opponent = player.other();
    for delirium in [false, true] {
        let mut game = scenario(player);
        for name in ["Forest", "Jade-Cast Sentinel"] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        let threshold = put(
            &mut game,
            player,
            "Pacifism",
            if delirium {
                Zone::Graveyard
            } else {
                Zone::Exile
            },
        );
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let hand =
            ["Forest", "Island", "Swamp"].map(|name| put(&mut game, opponent, name, Zone::Hand));
        let spell = put(&mut game, player, "Let's Play a Game", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        let modes = game
            .legal_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::ChooseMode { .. }))
            .count();
        assert_eq!(modes, if delirium { 7 } else { 3 });
        if !delirium {
            reject(&mut game, Action::ChooseMode { mode: 6 });
        }
        act(
            &mut game,
            Action::ChooseMode {
                mode: if delirium { 6 } else { 2 },
            },
        );
        pay(&mut game);
        if delirium {
            game.move_object(threshold, Zone::Exile);
            pass_pair(&mut game);
            assert_eq!(stats(&game, enemy), (1, 1));
            assert_eq!(game.state.players[opponent.index()].life, 20);
            assert_eq!(game.actor(), Some(opponent));
            for card in [hand[0], hand[2]] {
                act(&mut game, Action::Discard { card });
            }
            assert_eq!(game.object_view(hand[0]).zone, Zone::Graveyard);
            assert_eq!(game.object_view(hand[1]).zone, Zone::Hand);
            assert_eq!(game.object_view(hand[2]).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(stats(&game, own), (2, 2));
        assert_eq!(stats(&game, enemy), if delirium { (1, 1) } else { (2, 2) });
        assert_eq!(game.state.players[player.index()].life, 23);
        assert_eq!(game.state.players[opponent.index()].life, 17);
    }
}

#[test]
fn trial_of_agony_lets_the_opponent_choose_damage_and_falls_back_to_the_one_legal_target() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let [first, second, unrelated] =
        ["Gigantosaurus"; 3].map(|name| put(&mut game, opponent, name, Zone::Battlefield));
    let spell = put(&mut game, player, "Trial of Agony", Zone::Hand);
    announce(
        &mut game,
        spell,
        &[Target::Card(first), Target::Card(second)],
    );
    let mut stale = game.clone();
    stale.move_object(first, Zone::Exile);
    stale.move_object(first, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stale.object_view(second).damage, 5);
    assert!(
        !stale
            .object_view(second)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
    assert_eq!(stale.object_view(first).damage, 0);

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let choice = |card| Action::ChooseEffectObject {
        object: game.object_ref(card),
    };
    let invalid = choice(unrelated);
    reject(&mut game, invalid);
    let choice = Action::ChooseEffectObject {
        object: game.object_ref(first),
    };
    act(&mut game, choice);
    assert_eq!(game.object_view(first).damage, 5);
    assert_eq!(game.object_view(second).damage, 0);
    assert!(
        game.object_view(second)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
}

#[test]
fn untimely_malfunction_resolves_each_of_its_three_printed_modes() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let bear = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let giants = ["Hill Giant"; 2].map(|name| put(&mut game, opponent, name, Zone::Battlefield));
    let artifact_mode = put(&mut game, player, "Untimely Malfunction", Zone::Hand);
    mode(&mut game, artifact_mode, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    target(&mut game, Target::Card(artifact));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);

    let damage = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, damage, Some(Target::Player(opponent)));
    let change_mode = put(&mut game, player, "Untimely Malfunction", Zone::Hand);
    mode(&mut game, change_mode, 1);
    target(&mut game, Target::Spell(damage));
    pay(&mut game);
    pass_pair(&mut game);
    reject(&mut game, Action::FinishStackTargets);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(bear),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    let blocks_mode = put(&mut game, player, "Untimely Malfunction", Zone::Hand);
    mode(&mut game, blocks_mode, 2);
    for card in giants {
        target(&mut game, Target::Card(card));
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert!(giants.iter().all(|&card| {
        game.object_view(card)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    }));
}

#[test]
fn vengeful_possession_untaps_and_grants_haste_with_an_independent_discard_choice() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let spell = put(&mut game, player, "Vengeful Possession", Zone::Hand);
    let hand = put(&mut game, player, "Island", Zone::Hand);
    let top = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, spell, Some(Target::Card(creature)));
    pass_pair(&mut game);
    let taken = game.object_view(creature);
    assert_eq!(taken.controller, player);
    assert_eq!(taken.owner, opponent);
    assert!(!taken.tapped);
    assert!(taken.keywords.contains(Keywords::HASTE));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(top).zone, Zone::Library);
    assert_eq!(declined.object_view(hand).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: hand });
    assert_eq!(game.object_view(hand).zone, Zone::Graveyard);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    game.begin_step(Step::End);
    game.prepare_triggers();
    pass_pair(&mut game);
    let restored = game.object_view(creature);
    assert_eq!(restored.controller, opponent);
    assert!(!restored.keywords.contains(Keywords::HASTE));
}

#[test]
fn vile_mutilator_pays_a_creature_cost_and_requires_two_sequential_nontoken_sacrifices() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Vile Mutilator", Zone::Hand);
    let owned_artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    game.create_tokens(player, Card::from_catalogue_id(5258), 1, false);
    let owned_token = game.controlled_token_ids(player).next().unwrap();
    let enchantment = put(&mut game, opponent, "Fear of the Dark", Zone::Battlefield);
    assert_eq!(game.object_view(enchantment).kinds, 1 | 32);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.create_tokens(opponent, Card::from_catalogue_id(5258), 1, false);
    let token = game.controlled_token_ids(opponent).next().unwrap();
    act(&mut game, Action::AnnounceCast { card: source });
    let invalid = cost_object(&game, owned_artifact);
    reject(&mut game, invalid);
    let cost = cost_object(&game, owned_token);
    act(&mut game, cost);
    pay(&mut game);
    assert_eq!(game.object_view(owned_token).zone, Zone::Outside);
    pass_pair(&mut game);
    let keywords = game.object_view(source).keywords;
    assert!(keywords.contains(Keywords::FLYING.union(Keywords::TRAMPLE)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let invalid = sacrifice(&game, creature);
    reject(&mut game, invalid);
    let paid = sacrifice(&game, enchantment);
    act(&mut game, paid);
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    let invalid = sacrifice(&game, token);
    reject(&mut game, invalid);
    let paid = sacrifice(&game, creature);
    act(&mut game, paid);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
}
