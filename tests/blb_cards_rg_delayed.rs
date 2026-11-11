//! Card-level play permissions, exact delayed copies, and the two R/G Seasons.
mod support;

use rustmage as _;
use rustmage_core::PermanentType;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain(game: &mut Game, decision: impl Fn(&Action) -> bool) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. }) || decision(action)
        });
    }
    panic!("Bloomburrow effects did not settle");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn season(game: &mut Game, player: Player, name: &str, mode: u16) {
    let card = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceCast { card });
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| { matches!(action, Action::ChooseMode { .. }) })
            .count(),
        16
    );
    act(game, Action::ChooseMode { mode });
    pay(game);
}

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
    drain(game, |_| false);
}

#[test]
fn dragonhawk_grants_play_until_the_end_step_and_attacks_into_a_new_batch() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Quaketusk Boar", Zone::Battlefield);
    let top = put(&mut game, player, "Forest", Zone::Library);
    let hawk = put(&mut game, player, "Dragonhawk, Fate's Tempest", Zone::Hand);
    cast(&mut game, hawk, None);
    drain(&mut game, |_| false);
    assert!(game.keywords(hawk).contains(Keywords::FLYING));
    let permissions = game.observe(player).exile_permissions;
    assert_eq!(permissions.len(), 2);
    assert!(permissions.iter().all(|row| {
        row.player == player && row.expiry == ExilePlayExpiry::StartOfNextEndStep(player)
    }));
    act(&mut game, Action::PlayLand { card: top });
    end_step(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert!(game.observe(player).exile_permissions.is_empty());

    game.finish_cleanup();
    game.finish_cleanup();
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = player;
    act(&mut game, Action::Attack { card: hawk });
    act(&mut game, Action::FinishAttackers);
    drain(&mut game, |_| false);
    assert_eq!(game.observe(player).exile_permissions.len(), 2);
}

#[test]
fn festival_graveyard_casts_pay_life_and_its_resolving_sacrifice_exiles_it() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let festival = put(&mut game, player, "Festival of Embers", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let cast = Action::AnnounceGraveyardWithLife {
        object: game.object_ref(shock),
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        life: 1,
        entry_counter: None,
    };
    let mut off_turn = game.clone();
    off_turn.state.active = opponent;
    reject(&mut off_turn, cast.clone());
    act(&mut game, cast);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);

    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, .. }
            | Action::AnnounceActivate { card, ability: 0, .. } if *card == festival)
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(festival).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(festival).zone, Zone::Exile);
}

#[test]
fn an_otter_cast_can_use_harnesser_once_and_stormsplitter_copies_only_spells() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Harnesser of Storms", Zone::Battlefield);
    let splitter = put(&mut game, player, "Stormsplitter", Zone::Battlefield);
    let otter = put(&mut game, player, "Kindlespark Duo", Zone::Hand);
    cast(&mut game, otter, None);
    drain(&mut game, |action| {
        *action == Action::ChooseOptional { accept: false }
    });
    assert!(game.observe(player).exile_permissions.is_empty());
    assert_eq!(game.controlled_token_ids(player).count(), 0);

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    drain(&mut game, |_| false);
    let child = game.controlled_token_ids(player).next().unwrap();
    assert_eq!(
        game.object_view(child).card,
        game.object_view(splitter).card
    );
    assert!(game.keywords(child).contains(Keywords::HASTE));
    assert!(game.observe(player).exile_permissions.is_empty());
    end_step(&mut game);
    assert!(!game.on_battlefield(child));
    assert!(game.on_battlefield(splitter));

    game.finish_cleanup();
    game.finish_cleanup();
    game.state.players[player.index()].mana = [8; 6];
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    drain(&mut game, |action| {
        *action == Action::ChooseOptional { accept: true }
    });
    assert_eq!(game.observe(player).exile_permissions.len(), 1);
}

#[test]
fn common_good_spends_two_per_x_and_grants_indestructible_to_all_current_tokens() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.create_tokens(player, Card::from_catalogue_id(5246), 1, false);
    assert!(!game.advance_entries());
    let original = game.controlled_token_ids(player).next().unwrap();
    let common = put(&mut game, player, "For the Common Good", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 4];
    act(&mut game, Action::AnnounceCast { card: common });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, target(original));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let tokens: Vec<_> = game.controlled_token_ids(player).collect();
    assert_eq!(tokens.len(), 3);
    assert!(
        tokens
            .iter()
            .all(|&token| game.keywords(token).contains(Keywords::INDESTRUCTIBLE))
    );
    assert_eq!(game.state.players[player.index()].life, 23);
    game.finish_cleanup();
    assert!(
        tokens
            .iter()
            .all(|&token| game.keywords(token).contains(Keywords::INDESTRUCTIBLE))
    );
    game.finish_cleanup();
    assert!(
        tokens
            .iter()
            .all(|&token| !game.keywords(token).contains(Keywords::INDESTRUCTIBLE))
    );
}

#[test]
fn seasons_execute_repeated_modes_in_printed_order() {
    let player = Player::P1;
    let opponent = player.other();
    let mut bold = scenario(player);
    season(&mut bold, player, "Season of the Bold", 11);
    settle(&mut bold);
    assert_eq!(bold.observe(player).exile_permissions.len(), 4);
    let treasure = bold.controlled_token_ids(player).next().unwrap();
    assert!(bold.has_subtype(treasure, "Treasure"));
    assert!(bold.object_view(treasure).tapped);

    bold.state.players[player.index()].mana = [8; 6];
    let target_creature = put(&mut bold, opponent, "Grizzly Bears", Zone::Battlefield);
    season(&mut bold, player, "Season of the Bold", 15);
    settle(&mut bold);
    assert!(bold.observe(player).turn_callbacks.is_some());
    let growth = put(&mut bold, player, "Giant Growth", Zone::Hand);
    cast(&mut bold, growth, Some(Target::Card(target_creature)));
    drain(&mut bold, |action| *action == target(target_creature));
    assert_eq!(bold.object_view(target_creature).zone, Zone::Graveyard);

    let mut gathering = scenario(player);
    let creature = put(&mut gathering, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut gathering, player, "Candy Trail", Zone::Battlefield);
    let enchantment = put(
        &mut gathering,
        opponent,
        "Doubling Season",
        Zone::Battlefield,
    );
    season(&mut gathering, player, "Season of Gathering", 11);
    let pick = Action::ChooseEffectObject {
        object: gathering.object_ref(creature),
    };
    drain(&mut gathering, |action| {
        *action == pick
            || *action
                == Action::ChoosePermanentType {
                    permanent_type: PermanentType::Artifact,
                }
    });
    assert_eq!(gathering.counter_count(creature, CounterKind::PlusOne), 1);
    assert!(
        gathering
            .keywords(creature)
            .contains(Keywords::TRAMPLE.union(Keywords::VIGILANCE))
    );
    assert_eq!(gathering.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(gathering.object_view(enchantment).zone, Zone::Battlefield);

    gathering.state.players[player.index()].mana = [8; 6];
    season(&mut gathering, player, "Season of Gathering", 14);
    drain(&mut gathering, |action| *action == pick);
    assert_eq!(gathering.counter_count(creature, CounterKind::PlusOne), 3);
    assert_eq!(
        gathering.state.players[player.index()].cards_drawn_this_turn,
        5
    );
}
