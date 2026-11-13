//! Authored combat checks, damage, death, and graveyard actions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn order_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

#[test]
fn fear_of_the_dark_checks_the_defenders_creature_types_twice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let fear = put(&mut game, player, "Fear of the Dark", Zone::Battlefield);
    let glimmer = put(&mut game, player, "Enduring Curiosity", Zone::Battlefield);
    let exact = game.object_ref(glimmer);
    let attack = |game: &mut Game| {
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(game, Action::Attack { card: fear });
        act(game, Action::FinishAttackers);
    };
    let mut blocked_at_declaration = game.clone();
    blocked_at_declaration.change_controller(exact, opponent);
    attack(&mut blocked_at_declaration);
    assert!(blocked_at_declaration.state.stack.is_empty());

    attack(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().event_player,
        Some(opponent)
    );
    let mut blocked_at_resolution = game.clone();
    blocked_at_resolution.change_controller(exact, opponent);
    settle(&mut blocked_at_resolution);
    let granted = Keywords::MENACE.union(Keywords::DEATHTOUCH);
    assert!(
        !blocked_at_resolution
            .object_view(fear)
            .keywords
            .contains(Keywords::MENACE)
    );
    assert!(
        !blocked_at_resolution
            .object_view(fear)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    settle(&mut game);
    assert!(game.object_view(fear).keywords.contains(granted));
}

#[test]
fn impossible_inferno_uses_live_delirium_and_gives_a_land_play_permission() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    for name in ["Forest", "Jade-Cast Sentinel"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let enchantment = put(&mut game, player, "Pacifism", Zone::Graveyard);
    let top = put(&mut game, player, "Island", Zone::Library);
    let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let spell = put(&mut game, player, "Impossible Inferno", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(victim)));
    let mut no_delirium = game.clone();
    no_delirium.move_object(enchantment, Zone::Exile);
    settle(&mut no_delirium);
    assert_eq!(no_delirium.object_view(victim).damage, 6);
    assert_eq!(no_delirium.object_view(top).zone, Zone::Library);
    let mut no_target = game.clone();
    no_target.move_object(victim, Zone::Exile);
    settle(&mut no_target);
    assert_eq!(no_target.object_view(top).zone, Zone::Library);

    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 6);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    let exact = game.object_ref(top);
    assert!(
        game.observe(player)
            .exile_permissions
            .iter()
            .any(|permission| {
                permission.object == exact && permission.expiry == ExilePlayExpiry::EndOfNextTurn
            })
    );
    act(&mut game, Action::PlayLand { card: top });
    assert_eq!(game.object_view(top).zone, Zone::Battlefield);
}

#[test]
fn innocuous_rat_uses_its_predeath_controllers_private_manifest_choice() {
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, owner, "Innocuous Rat", Zone::Battlefield);
    game.change_controller(game.object_ref(source), player);
    let other = put(&mut game, player, "Forest", Zone::Library);
    let face_down = put(&mut game, player, "Bear Cub", Zone::Library);
    let other_top = put(&mut game, owner, "Swamp", Zone::Library);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.last().unwrap().controller, player);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(owner).manifest_choice.is_none());
    let wrong_library = Action::ChooseManifest {
        object: game.object_ref(other_top),
    };
    reject(&mut game, wrong_library);
    let object = game.object_ref(face_down);
    act(&mut game, Action::ChooseManifest { object });
    let manifested = game.object_view(face_down);
    assert_eq!(manifested.zone, Zone::Battlefield);
    assert_eq!(manifested.controller, player);
    assert_eq!(stats(&game, face_down), (2, 2));
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other_top).zone, Zone::Library);
}

#[test]
fn ragged_playmate_checks_current_power_at_announcement_and_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Ragged Playmate", Zone::Battlefield);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let action = announce_activation(source);
    let mut sick = game.clone();
    sick.state.objects[source as usize].summoning_sick = true;
    reject(&mut sick, action.clone());
    act(&mut game, action);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(large),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(small),
        },
    );
    pay(&mut game);
    assert!(game.object_view(source).tapped);
    let mut grew_early = game.clone();
    grew_early.add_counter(small, CounterKind::PlusOne, 1);
    settle(&mut grew_early);
    assert!(
        !grew_early
            .object_view(small)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    settle(&mut game);
    game.add_counter(small, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, small), (3, 3));
    assert!(
        game.object_view(small)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
}

#[test]
fn razorkin_needlehead_counts_every_opposing_draw_and_first_strike_follows_control() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Razorkin Needlehead", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    game.draw_many(player, 1);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.draw_many(opponent, 2);
    game.prepare_triggers();
    order_triggers(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    assert!(
        game.state
            .stack
            .iter()
            .all(|trigger| trigger.draw_event.unwrap().player == opponent)
    );
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);

    game.change_controller(game.object_ref(source), opponent);
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    game.state.active = opponent;
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    game.draw_many(player, 1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
}

#[test]
fn resurrected_cultist_is_announced_with_delirium_and_returns_with_finality() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Resurrected Cultist", Zone::Graveyard);
    for name in ["Forest", "Candy Trail"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let action = announce_activation(source);
    reject(&mut game, action.clone());
    let threshold = put(&mut game, player, "Pacifism", Zone::Graveyard);
    let mut off_main = game.clone();
    off_main.begin_step(Step::Upkeep);
    reject(&mut off_main, action.clone());
    act(&mut game, action);
    pay(&mut game);
    let exact = game.object_ref(source);
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(source).zone, Zone::Graveyard);
    game.move_object(threshold, Zone::Exile);
    settle(&mut game);
    assert_ne!(game.object_ref(source), exact);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(source, CounterKind::Finality), 1);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
}

#[test]
fn waltz_of_rage_observes_own_creature_and_token_deaths_from_its_damage_and_later() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::from_catalogue_id(5258), 1, false);
    let tops =
        ["Island", "Mountain", "Forest"].map(|name| put(&mut game, player, name, Zone::Library));
    let spell = put(&mut game, player, "Waltz of Rage", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(dealer)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(dealer).damage, 0);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(game.object_view(token).zone, Zone::Outside);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(tops[0]).zone, Zone::Library);
    for card in &tops[1..] {
        assert_eq!(game.object_view(*card).zone, Zone::Exile);
    }
    assert_eq!(game.observe(player).exile_permissions.len(), 2);

    let mut next_turn = game.clone();
    next_turn.finish_cleanup();
    assert_eq!(next_turn.state.turn, game.state.turn + 1);
    next_turn.move_object(dealer, Zone::Graveyard);
    next_turn.prepare_triggers();
    assert!(next_turn.state.stack.is_empty());
    game.move_object(dealer, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(tops[0]).zone, Zone::Exile);
    assert_eq!(game.observe(player).exile_permissions.len(), 3);
}
