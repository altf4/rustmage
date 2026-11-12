//! Exact offspring recipes and Bloomburrow's first own-target triggers.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain(game: &mut Game, decision: impl Fn(&Action) -> bool) {
    for _ in 0..48 {
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
    panic!("card triggers did not settle");
}

fn offspring(
    game: &mut Game,
    player: Player,
    name: &str,
    decision: impl Fn(&Action) -> bool,
) -> [CardId; 2] {
    let parent = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceCast { card: parent });
    act(game, Action::ChooseOffspring { pay: true });
    pay(game);
    drain(game, decision);
    let definition = Card::from_name(name).unwrap();
    let child = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.card == definition
        })
        .unwrap();
    assert_eq!(child.controller, player);
    assert_eq!(child.copy_recipe.unwrap().exceptions.base, Some((1, 1)));
    [parent, child.id]
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn coruscation_and_steampath_offspring_retain_their_damage_triggers() {
    let player = Player::P0;
    let opponent = player.other();
    let mut mage = scenario(player);
    let [parent, child] = offspring(&mut mage, player, "Coruscation Mage", |_| false);
    assert_eq!(stats(&mage, parent), (2, 2));
    assert_eq!(stats(&mage, child), (1, 1));
    let creature = put(&mut mage, player, "Bear Cub", Zone::Hand);
    cast(&mut mage, creature, None);
    settle(&mut mage);
    assert_eq!(mage.state.players[opponent.index()].life, 20);
    let growth = put(&mut mage, player, "Giant Growth", Zone::Hand);
    cast(&mut mage, growth, Some(Target::Card(parent)));
    drain(&mut mage, |_| false);
    assert_eq!(mage.state.players[opponent.index()].life, 18);

    let mut charger = scenario(player);
    let [parent, child] = offspring(&mut charger, player, "Steampath Charger", |_| false);
    for source in [child, parent] {
        let shock = put(&mut charger, player, "Shock", Zone::Hand);
        cast(&mut charger, shock, Some(Target::Card(source)));
        drain(&mut charger, |action| {
            *action
                == Action::ChooseTarget {
                    target: Target::Player(opponent),
                }
        });
    }
    assert_eq!(charger.state.players[opponent.index()].life, 18);
}

#[test]
fn manifold_and_pawpatch_offspring_each_have_their_own_targeted_trigger() {
    let player = Player::P1;
    let opponent = player.other();
    let mut mouse = scenario(player);
    let [parent, child] = offspring(&mut mouse, player, "Manifold Mouse", |_| false);
    mouse.begin_step(Step::BeginCombat);
    mouse.prepare_triggers();
    let mut chosen = false;
    for _ in 0..36 {
        if mouse.state.stack.is_empty() && mouse.state.pending == Pending::Priority {
            break;
        }
        let index = u16::from(chosen);
        choose(&mut mouse, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
                || *action == target(child)
                || *action == Action::ChooseKeyword { index }
        });
        chosen |= mouse.keywords(child).contains(Keywords::DOUBLE_STRIKE);
    }
    assert!(!mouse.keywords(parent).contains(Keywords::DOUBLE_STRIKE));
    assert!(
        mouse
            .keywords(child)
            .contains(Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE))
    );

    let mut rabbit = scenario(player);
    let [parent, child] = offspring(&mut rabbit, player, "Pawpatch Recruit", |_| false);
    assert!(rabbit.keywords(parent).contains(Keywords::TRAMPLE));
    assert!(rabbit.keywords(child).contains(Keywords::TRAMPLE));
    let marked = put(&mut rabbit, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut rabbit, player, "Bear Cub", Zone::Battlefield);
    let growth = put(&mut rabbit, opponent, "Giant Growth", Zone::Hand);
    act(&mut rabbit, Action::Pass);
    cast(&mut rabbit, growth, Some(Target::Card(marked)));
    choose(&mut rabbit, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    reject(&mut rabbit, target(marked));
    drain(&mut rabbit, |action| *action == target(recipient));
    assert_eq!(rabbit.counter_count(marked, CounterKind::PlusOne), 0);
    assert_eq!(rabbit.counter_count(recipient, CounterKind::PlusOne), 2);
}

#[test]
fn bushy_rust_shield_and_tender_offspring_keep_their_printed_rules() {
    let player = Player::P0;
    let mut bushy = scenario(player);
    bushy.create_tokens(player, Card::from_catalogue_id(5246), 2, false);
    assert!(!bushy.advance_entries());
    let bodyguards = offspring(&mut bushy, player, "Bushy Bodyguard", |action| {
        matches!(
            action,
            Action::ChooseForageRoute {
                route: ForageRoute::Food
            } | Action::ChooseForage { .. }
        )
    });
    for bodyguard in bodyguards {
        assert_eq!(bushy.counter_count(bodyguard, CounterKind::PlusOne), 2);
    }

    let mut rampager = scenario(player);
    let attackers = offspring(&mut rampager, player, "Rust-Shield Rampager", |_| false);
    let small = put(
        &mut rampager,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let large = put(
        &mut rampager,
        player.other(),
        "Hill Giant",
        Zone::Battlefield,
    );
    for attacker in attackers {
        assert!(!rampager.can_block_pair(small, attacker));
        assert!(rampager.can_block_pair(large, attacker));
    }

    let mut tender = scenario(player);
    let [parent, child] = offspring(&mut tender, player, "Tender Wildguide", |_| false);
    assert!(!tender.legal_actions().iter().any(|action| {
        matches!(action, Action::ActivateMana { card, .. } if *card == parent || *card == child)
    }));
    tender.finish_cleanup();
    tender.finish_cleanup();
    let blue = [0, 1, 0, 0, 0, 0];
    choose(&mut tender, |action| {
        matches!(action, Action::ActivateMana { card, ability: 0, production, .. }
            if *card == parent && *production == blue)
    });
    assert_eq!(tender.state.players[player.index()].mana, blue);
    choose(
        &mut tender,
        |action| matches!(action, Action::Activate { card, ability: 1, .. } if *card == child),
    );
    if matches!(tender.state.pending, Pending::PayCosts { .. }) {
        pay(&mut tender);
    }
    settle(&mut tender);
    assert_eq!(tender.counter_count(child, CounterKind::PlusOne), 1);
    assert_eq!(tender.counter_count(parent, CounterKind::PlusOne), 0);
}

#[test]
fn valiant_triggers_only_for_the_first_own_target_and_heartfire_uses_death_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ember = put(
        &mut game,
        player,
        "Emberheart Challenger",
        Zone::Battlefield,
    );
    let hero = put(&mut game, player, "Heartfire Hero", Zone::Battlefield);
    let scribe = put(&mut game, player, "Whiskerquill Scribe", Zone::Battlefield);
    assert!(game.keywords(ember).contains(Keywords::HASTE));
    assert!(!game.object_view(hero).card.is_standard_legal());

    let top = game.object_ref(game.top_library_card(player).unwrap());
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(ember)));
    drain(&mut game, |_| false);
    assert_eq!(stats(&game, ember), (6, 6));
    assert_eq!(game.object_view(top.card).zone, Zone::Exile);
    assert!(
        game.observe(player)
            .exile_permissions
            .iter()
            .any(|permission| {
                permission.object == game.object_ref(top.card) && permission.player == player
            })
    );

    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    for _ in 0..2 {
        let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, growth, Some(Target::Card(scribe)));
        drain(&mut game, |action| {
            *action == Action::Discard { card: discarded }
        });
    }
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let growth = put(&mut game, opponent, "Giant Growth", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, growth, Some(Target::Card(hero)));
    settle(&mut game);
    assert_eq!(game.counter_count(hero, CounterKind::PlusOne), 0);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(hero)));
    drain(&mut game, |_| false);
    assert_eq!(game.counter_count(hero, CounterKind::PlusOne), 1);
    game.finish_cleanup();
    game.state.players[opponent.index()].mana = [8; 6];
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(hero)));
    settle(&mut game);
    assert_eq!(game.object_view(hero).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}
