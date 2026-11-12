//! Authored Duskmourn discard, additional-cost, and reflexive trigger interactions.
mod support;

use rustmage::AdditionalCostRoute;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter_creature(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, None);
    pass_pair(game);
    card
}

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn hand_card(game: &Game, card: CardId) -> Action {
    Action::ChooseHandCard {
        object: Some(game.object_ref(card)),
    }
}

fn cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

#[test]
fn betrayers_bargain_accepts_both_costs_and_redirects_a_later_death_after_prevention() {
    let player = Player::P1;
    let opponent = player.other();
    let mut parent = scenario(player);
    let spell = put(&mut parent, player, "Betrayer's Bargain", Zone::Hand);
    let victim = put(&mut parent, opponent, "Hill Giant", Zone::Battlefield);
    let enchantment = put(
        &mut parent,
        player,
        "Deeproot Pilgrimage",
        Zone::Battlefield,
    );
    let artifact = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    parent.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];

    for route in [AdditionalCostRoute::Nonmana, AdditionalCostRoute::Mana] {
        let mut game = parent.clone();
        let mana = route == AdditionalCostRoute::Mana;
        if mana {
            game.add_counter(victim, CounterKind::Shield, 1);
        }
        act(&mut game, Action::AnnounceCast { card: spell });
        let cost = game
            .observe(player)
            .cost_choices
            .iter()
            .find(|cost| cost.additional_cost_route == Some(route))
            .unwrap()
            .index;
        act(&mut game, Action::ChooseSpellCost { cost });
        act(&mut game, target(Target::Card(victim)));
        if !mana {
            let illegal = cost_object(&game, artifact);
            reject(&mut game, illegal);
            let selected = cost_object(&game, enchantment);
            act(&mut game, selected);
        }
        if matches!(game.state.pending, Pending::CastCosts { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            });
        }
        let payment = [0, 0, 0, 1, 0, if mana { 3 } else { 1 }];
        act(&mut game, Action::PayCosts { payment });
        assert_eq!(
            game.object_view(enchantment).zone,
            if mana {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        settle(&mut game);
        if mana {
            assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(game.object_view(victim).damage, 0);
            assert_eq!(game.counter_count(victim, CounterKind::Shield), 0);
            game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
            let removal = put(&mut game, player, "Murder", Zone::Hand);
            cast(&mut game, removal, Some(Target::Card(victim)));
            settle(&mut game);
        }
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    }
}

#[test]
fn grab_the_prize_requires_a_discard_and_damages_only_for_the_paid_nonland() {
    let player = Player::P1;
    let opponent = player.other();
    for (name, life) in [("Forest", 20), ("Shock", 18)] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Grab the Prize", Zone::Hand);
        let cast = Action::AnnounceCast { card: spell };
        reject(&mut game, cast.clone());
        let discarded = put(&mut game, player, name, Zone::Hand);
        put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
        act(&mut game, cast);
        let selection = cost_object(&game, discarded);
        act(&mut game, selection);
        assert_eq!(game.object_view(discarded).zone, Zone::Hand);
        assert!(game.observe(opponent).additional_cost.is_none());
        pay(&mut game);
        assert_eq!(game.object_view(discarded).zone, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        assert_eq!(game.state.players[opponent.index()].life, life);
    }
}

#[test]
fn irreverent_gremlin_can_decline_then_use_its_small_creature_trigger_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gremlin = enter_creature(&mut game, player, "Irreverent Gremlin");
    settle(&mut game);
    assert_eq!(stats(&game, gremlin), (2, 2));
    assert!(
        game.object_view(gremlin)
            .keywords
            .contains(Keywords::MENACE)
    );
    let discard = put(&mut game, player, "Island", Zone::Hand);
    enter_creature(&mut game, player, "Hill Giant");
    assert!(game.state.stack.is_empty());

    enter_creature(&mut game, player, "Bear Cub");
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.object_view(discard).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    enter_creature(&mut game, player, "Bear Cub");
    pass_pair(&mut game);
    let selection = cost_object(&game, discard);
    act(&mut game, selection);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    enter_creature(&mut game, player, "Bear Cub");
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn fanatic_of_the_harrowing_draws_for_its_controllers_discard_only() {
    let player = Player::P0;
    let opponent = player.other();
    for own_card in [false, true] {
        let mut game = scenario(player);
        let own = own_card.then(|| put(&mut game, player, "Forest", Zone::Hand));
        let enemy = put(&mut game, opponent, "Opt", Zone::Hand);
        let fanatic = enter_creature(&mut game, player, "Fanatic of the Harrowing");
        assert_eq!(stats(&game, fanatic), (2, 2));
        pass_pair(&mut game);
        if let Some(own) = own {
            act(&mut game, Action::Discard { card: own });
            assert_eq!(game.object_view(own).zone, Zone::Hand);
            assert!(
                !game
                    .observe(opponent)
                    .objects
                    .iter()
                    .any(|view| view.id == own)
            );
        }
        assert_eq!(game.actor(), Some(opponent));
        act(&mut game, Action::Discard { card: enemy });
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        if let Some(own) = own {
            assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        }
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(own_card)
        );
        assert_eq!(
            game.state.players[opponent.index()].cards_drawn_this_turn,
            0
        );
    }
}

#[test]
fn cracked_skull_looks_privately_and_destroys_the_enchanted_creature_on_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, player, "Cracked Skull", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let spell = put(&mut game, opponent, "Opt", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(victim)));
    pass_pair(&mut game);
    assert!(
        game.legal_actions()
            .contains(&target(Target::Player(player)))
    );
    act(&mut game, target(Target::Player(opponent)));
    pass_pair(&mut game);

    let choice = game.observe(player).hand_choice.unwrap();
    assert_eq!(choice.inspected_cards.unwrap().len(), 2);
    assert_eq!(choice.candidates, [game.object_ref(spell)]);
    assert!(game.observe(opponent).hand_choice.is_none());
    let land_action = hand_card(&game, land);
    reject(&mut game, land_action);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseHandCard { object: None });
    assert_eq!(decline.object_view(spell).zone, Zone::Hand);
    let spell_action = hand_card(&game, spell);
    act(&mut game, spell_action);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    for seat in [player, opponent] {
        assert_eq!(game.state.players[seat.index()].cards_drawn_this_turn, 0);
    }

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(victim)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn miasma_demon_discards_before_choosing_up_to_that_many_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let keep = put(&mut game, player, "Shock", Zone::Hand);
    let first = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let third = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let demon = enter_creature(&mut game, player, "Miasma Demon");
    assert_eq!(stats(&game, demon), (5, 4));
    assert!(game.object_view(demon).keywords.contains(Keywords::FLYING));
    pass_pair(&mut game);
    assert!(game.observe(opponent).hand_choice.is_none());
    reject(&mut game, target(Target::Card(first)));
    let done = Action::ChooseHandCard { object: None };
    let mut decline = game.clone();
    act(&mut decline, done.clone());
    assert!(decline.state.stack.is_empty());
    assert_eq!(decline.object_view(land).zone, Zone::Hand);

    for card in [land, spell] {
        let selection = hand_card(&game, card);
        act(&mut game, selection);
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    act(&mut game, done);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(keep).zone, Zone::Hand);
    assert_eq!(
        game.observe(player)
            .announcing
            .unwrap()
            .reflexive_target_limit,
        Some(2)
    );
    assert!(game.legal_actions().contains(&target(Target::Card(demon))));
    act(&mut game, target(Target::Card(first)));
    act(&mut game, target(Target::Card(second)));
    reject(&mut game, target(Target::Card(third)));
    settle(&mut game);
    assert_eq!(stats(&game, first), (1, 1));
    assert_eq!(stats(&game, second), (1, 1));
    assert_eq!(stats(&game, third), (3, 3));
}
