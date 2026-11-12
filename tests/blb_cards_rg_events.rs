//! Attacks, ordinal casts, and live typal damage on the real cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn drain(game: &mut Game, decision: Option<Target>) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
                || decision.is_some_and(|chosen| *action == target(chosen))
        });
    }
    panic!("card triggers did not settle");
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = game.state.active;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn activation(action: &Action, source: CardId) -> bool {
    matches!(*action, Action::Activate { card, ability: 0, .. }
        | Action::AnnounceActivate { card, ability: 0, .. } if card == source)
}

fn activate(game: &mut Game, source: CardId, chosen: Option<Target>) {
    choose(game, |action| {
        activation(action, source)
            && match action {
                Action::Activate { target, .. } => *target == chosen,
                _ => true,
            }
    });
    if let Some(chosen) = chosen
        && matches!(game.state.pending, Pending::Targets { .. })
    {
        act(game, target(chosen));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    drain(game, None);
}

#[test]
fn captain_samples_power_in_combat_and_collector_retains_red_past_a_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    let captain = put(&mut game, player, "Brambleguard Captain", Zone::Battlefield);
    let collector = put(&mut game, player, "Brazen Collector", Zone::Battlefield);
    assert!(game.keywords(collector).contains(Keywords::FIRST_STRIKE));
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    act(&mut game, target(Target::Card(collector)));
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(captain)));
    drain(&mut game, None);
    assert_eq!(stats(&game, collector), (7, 1));

    game.state.players[player.index()].mana = [0; 6];
    attack(&mut game, &[collector]);
    drain(&mut game, None);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 1, 0, 0]);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::Blockers);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 1, 0, 0]);
    game.finish_cleanup();
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn lizard_attack_and_tap_damage_each_get_one_flamecaller_bonus() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Valley Flamecaller", Zone::Battlefield);
    let claw = put(&mut game, player, "Hired Claw", Zone::Battlefield);
    let duo = put(&mut game, player, "Kindlespark Duo", Zone::Battlefield);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| activation(action, claw))
    );
    attack(&mut game, &[claw, duo]);
    drain(&mut game, Some(Target::Player(opponent)));
    assert_eq!(game.state.players[opponent.index()].life, 18);
    activate(&mut game, claw, None);
    assert_eq!(game.counter_count(claw, CounterKind::PlusOne), 1);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| activation(action, claw))
    );

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    drain(&mut game, None);
    assert!(!game.object_view(duo).tapped);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    activate(&mut game, duo, Some(Target::Player(opponent)));
    assert!(game.object_view(duo).tapped);
    assert_eq!(game.state.players[opponent.index()].life, 14);
}

#[test]
fn hearthborn_observes_the_second_spell_of_the_other_player() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let battler = put(&mut game, player, "Hearthborn Battler", Zone::Battlefield);
    assert!(game.keywords(battler).contains(Keywords::HASTE));
    for expected_life in [20, 18, 18] {
        if game.actor() != Some(opponent) {
            act(&mut game, Action::Pass);
        }
        let shock = put(&mut game, opponent, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player)));
        drain(&mut game, Some(Target::Player(opponent)));
        assert_eq!(game.state.players[opponent.index()].life, expected_life);
    }
}

#[test]
fn rallier_haste_and_herbalist_token_bonus_apply_to_the_selected_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rallier = put(&mut game, player, "Raccoon Rallier", Zone::Battlefield);
    put(&mut game, player, "Hazardroot Herbalist", Zone::Battlefield);
    let boar = put(&mut game, player, "Quaketusk Boar", Zone::Hand);
    cast(&mut game, boar, None);
    settle(&mut game);
    assert!(
        game.keywords(boar).contains(
            Keywords::HASTE
                .union(Keywords::REACH)
                .union(Keywords::TRAMPLE)
        )
    );
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, giant, None);
    settle(&mut game);
    let fodder = put(&mut game, player, "Dragon Fodder", Zone::Hand);
    cast(&mut game, fodder, None);
    settle(&mut game);
    let token = game.controlled_token_ids(player).next().unwrap();
    activate(&mut game, rallier, Some(Target::Card(giant)));
    assert!(game.keywords(giant).contains(Keywords::HASTE));
    attack(&mut game, &[giant, boar]);
    drain(&mut game, Some(Target::Card(token)));
    assert_eq!(stats(&game, token), (2, 1));
    assert!(game.keywords(token).contains(Keywords::DEATHTOUCH));
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| activation(action, rallier))
    );
}

#[test]
fn lynx_uses_each_nonbasic_count_and_moose_can_be_cast_off_turn() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Hidden Grotto", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut game, opponent, "Hidden Grotto", Zone::Battlefield);
    }
    put(&mut game, player, "Mountain", Zone::Battlefield);
    let lynx = put(&mut game, player, "Sunspine Lynx", Zone::Hand);
    cast(&mut game, lynx, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    game.gain_life(player, 2);
    game.gain_life(opponent, 2);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[opponent.index()].life, 18);

    game.state.active = opponent;
    let moose = put(&mut game, player, "Galewind Moose", Zone::Hand);
    cast(&mut game, moose, None);
    settle(&mut game);
    assert_eq!(stats(&game, moose), (6, 6));
    assert!(
        game.keywords(moose).contains(
            Keywords::FLASH
                .union(Keywords::REACH)
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
        )
    );
}
