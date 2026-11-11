//! Inherited Bloomburrow cast, life-history, entry and loyalty payoffs.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn idle(game: &mut Game) {
    for _ in 0..40 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("unresolved scenario choice in {:?}", game.state.pending);
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn pay_if_needed(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn dreamdew_entrancer_stuns_any_chosen_creature_and_draws_only_for_yours() {
    let player = Player::P0;
    let mut game = scenario(player);
    let entrancer = put(&mut game, player, "Dreamdew Entrancer", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, entrancer, None);
    pass_pair(&mut game);
    assert!(game.keywords(entrancer).contains(Keywords::REACH));

    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    idle(&mut no_target);
    assert_eq!(
        no_target.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    for (card, draws) in [(own, 2), (enemy, 0)] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
        idle(&mut branch);
        assert!(branch.object_view(card).tapped);
        assert_eq!(branch.counter_count(card, CounterKind::Stun), 3);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}

#[test]
fn lunar_convocation_has_independent_life_history_gates_and_an_actual_paid_draw() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Lunar Convocation", Zone::Battlefield);

    let mut gained = game.clone();
    gained.gain_life(player, 2);
    gained.begin_step(Step::End);
    gained.give_priority();
    gained.prepare_triggers();
    idle(&mut gained);
    assert_eq!(gained.state.players[opponent.index()].life, 19);
    assert!(gained.controlled_token_ids(player).next().is_none());

    game.gain_life(player, 2);
    act(&mut game, activate(enchantment, 0));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 1, 0, 0, 1],
        },
    );
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    idle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    let bat = game.controlled_token_ids(player).next().unwrap();
    assert!(game.has_subtype(bat, "Bat") && game.keywords(bat).contains(Keywords::FLYING));
    assert_eq!(stats(&game, bat), (1, 1));
}

#[test]
fn stormcatch_reduces_only_instants_or_sorceries_and_angler_keeps_its_cast_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Stormcatch Mentor", Zone::Battlefield);
    let angler = put(&mut game, player, "Tempest Angler", Zone::Battlefield);
    let divination = put(&mut game, player, "Divination", Zone::Hand);
    let equipment = put(&mut game, player, "Short Bow", Zone::Hand);
    assert!(game.keywords(mentor).contains(Keywords::HASTE));
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    cast(&mut game, divination, None);
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(stats(&game, mentor), (2, 2));
    assert_eq!(stats(&game, angler), (3, 3));

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: equipment });
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(&mut game, equipment, None);
    idle(&mut game);
    assert_eq!(stats(&game, mentor), (3, 3));
    assert_eq!(game.counter_count(angler, CounterKind::PlusOne), 2);
    game.finish_cleanup();
    assert_eq!(stats(&game, mentor), (1, 1));
    assert_eq!(stats(&game, angler), (4, 4));
}

#[test]
fn ral_uses_all_three_loyalty_abilities_and_the_emblem_copies_later_instants() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let ral = put(&mut game, player, "Ral, Crackling Wit", Zone::Hand);
    cast(&mut game, ral, None);
    idle(&mut game);
    assert_eq!(game.counter_count(ral, CounterKind::Loyalty), 4);
    reject(&mut game, activate(ral, 2));

    let mut plus = game.clone();
    act(&mut plus, activate(ral, 0));
    pay_if_needed(&mut plus);
    idle(&mut plus);
    assert_eq!(plus.counter_count(ral, CounterKind::Loyalty), 5);
    reject(&mut plus, activate(ral, 1));
    let otter = plus.controlled_token_ids(player).next().unwrap();
    assert!(plus.has_subtype(otter, "Otter"));
    let bolt = put(&mut plus, player, "Shock", Zone::Hand);
    cast(&mut plus, bolt, Some(Target::Player(opponent)));
    idle(&mut plus);
    assert_eq!(plus.counter_count(ral, CounterKind::Loyalty), 6);
    assert_eq!(stats(&plus, otter), (2, 2));

    let mut draw = game.clone();
    act(&mut draw, activate(ral, 1));
    pay_if_needed(&mut draw);
    pass_pair(&mut draw);
    assert_eq!(draw.counter_count(ral, CounterKind::Loyalty), 1);
    assert_eq!(draw.state.players[player.index()].cards_drawn_this_turn, 3);
    for _ in 0..2 {
        choose(&mut draw, |action| matches!(action, Action::Discard { .. }));
    }
    assert_eq!(
        draw.state
            .objects
            .iter()
            .filter(|object| object.owner == player && object.zone == Zone::Hand)
            .count(),
        1
    );

    game.add_counter(ral, CounterKind::Loyalty, 6);
    act(&mut game, activate(ral, 2));
    pay_if_needed(&mut game);
    idle(&mut game);
    assert_eq!(game.object_view(ral).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
    assert!(
        game.state
            .objects
            .iter()
            .any(|object| object.zone == Zone::Command
                && object.card == Card::from_catalogue_id(5468))
    );
    let equipment = put(&mut game, player, "Short Bow", Zone::Hand);
    cast(&mut game, equipment, None);
    idle(&mut game);
    let bolt = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Player(opponent)));
    idle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 14);
}
