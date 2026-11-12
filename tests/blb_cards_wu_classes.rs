mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn progress(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::Pass | Action::PutTrigger { .. })
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..24 {
        if game.legal_actions().contains(&action) {
            return;
        }
        progress(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn drain(game: &mut Game, recipient: Option<CardId>) -> usize {
    let mut scries = 0;
    for _ in 0..48 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return scries;
        }
        if let Some(card) = recipient {
            let action = target(card);
            if game.legal_actions().contains(&action) {
                act(game, action);
                continue;
            }
        }
        if let Some(action) = game.legal_actions().into_iter().find(|a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        }) {
            scries += 1;
            act(game, action);
        } else {
            progress(game);
        }
    }
    panic!("Class and batch test did not settle");
}

fn activate_untargeted(game: &mut Game, card: CardId, ability: u16) {
    choose(game, |a| {
        matches!(a,
            Action::Activate { card: id, ability: index, .. }
            | Action::AnnounceActivate { card: id, ability: index, .. }
            if *id == card && *index == ability
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn tokens(game: &Game, player: Player, subtype: &str) -> Vec<CardId> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.is_token()
                && o.controller == player
                && o.has_subtype(subtype)
        })
        .map(|o| o.id)
        .collect()
}

#[test]
fn builders_talent_creates_white_defender_counts_a_batch_and_returns_a_permanent() {
    let player = Player::P0;
    let mut game = scenario(player);
    let class = put(&mut game, player, "Builder's Talent", Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let wrong = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    cast(&mut game, class, None);
    drain(&mut game, None);
    let walls = tokens(&game, player, "Wall");
    assert_eq!(walls.len(), 1);
    let wall = walls[0];
    let view = game.object_view(wall);
    assert_eq!(view.colors, 1);
    assert!(view.keywords.contains(Keywords::DEFENDER));
    assert_eq!(stats(&game, wall), (0, 4));

    game.create_tokens(player, Card(5246), 1, false);
    drain(&mut game, None);
    assert_eq!(game.counter_count(wall, CounterKind::PlusOne), 0);
    activate_untargeted(&mut game, class, 0);
    drain(&mut game, None);
    assert_eq!(game.state.objects[class as usize].class_level, 2);
    game.create_tokens(player, Card(5246), 2, false);
    game.prepare_triggers();
    drain(&mut game, Some(wall));
    assert_eq!(game.counter_count(wall, CounterKind::PlusOne), 1);

    activate_untargeted(&mut game, class, 1);
    until(&mut game, target(boots));
    reject(&mut game, target(wrong));
    act(&mut game, target(boots));
    drain(&mut game, Some(wall));
    assert_eq!(game.object_view(boots).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(wall, CounterKind::PlusOne), 2);
}

#[test]
fn caretaker_draws_once_copies_a_noncreature_token_and_boosts_only_creature_tokens() {
    let player = Player::P1;
    let mut game = scenario(player);
    let class = put(&mut game, player, "Caretaker's Talent", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, class, None);
    drain(&mut game, None);
    game.create_tokens(player, Card(5285), 2, false);
    game.prepare_triggers();
    drain(&mut game, None);
    game.create_tokens(player, Card(5246), 1, false);
    game.prepare_triggers();
    drain(&mut game, None);
    let food = tokens(&game, player, "Food")[0];
    activate_untargeted(&mut game, class, 0);
    until(&mut game, target(food));
    reject(&mut game, target(creature));
    act(&mut game, target(food));
    drain(&mut game, None);
    assert_eq!(tokens(&game, player, "Food").len(), 2);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    activate_untargeted(&mut game, class, 1);
    drain(&mut game, None);
    for rabbit in tokens(&game, player, "Rabbit") {
        assert_eq!(stats(&game, rabbit), (3, 3));
    }
    assert_eq!(stats(&game, creature), (2, 2));
}

#[test]
fn carrot_cake_creates_and_scries_on_entry_and_sacrifice_then_gains_three() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cake = put(&mut game, player, "Carrot Cake", Zone::Hand);
    cast(&mut game, cake, None);
    assert_eq!(drain(&mut game, None), 1);
    assert_eq!(tokens(&game, player, "Rabbit").len(), 1);
    let mut destroy = game.clone();
    destroy.move_object(cake, Zone::Graveyard);
    drain(&mut destroy, None);
    assert_eq!(tokens(&destroy, player, "Rabbit").len(), 1);
    activate_untargeted(&mut game, cake, 0);
    assert_eq!(game.object_view(cake).zone, Zone::Graveyard);
    assert_eq!(drain(&mut game, None), 1);
    assert_eq!(tokens(&game, player, "Rabbit").len(), 2);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn harvestrite_counts_each_resolution_and_questcaller_counts_the_batch() {
    let player = Player::P0;
    let mut game = scenario(player);
    let quest = put(&mut game, player, "Valley Questcaller", Zone::Battlefield);
    let dual = put(&mut game, player, "Brave-Kin Duo", Zone::Battlefield);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&game, quest), (2, 3));
    assert_eq!(stats(&game, dual), (2, 2));
    let host = put(&mut game, player, "Harvestrite Host", Zone::Hand);
    cast(&mut game, host, None);
    assert_eq!(drain(&mut game, Some(recipient)), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    let rabbits = put(&mut game, player, "Hop to It", Zone::Hand);
    cast(&mut game, rabbits, None);
    assert_eq!(drain(&mut game, Some(recipient)), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, recipient), (6, 2));
    assert_eq!(tokens(&game, player, "Rabbit").len(), 3);
    for rabbit in tokens(&game, player, "Rabbit") {
        assert_eq!(stats(&game, rabbit), (2, 2));
    }
}

#[test]
fn jackdaw_uses_lesser_mana_value_for_the_dead_flyer_and_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jackdaw = put(&mut game, player, "Jackdaw Savior", Zone::Battlefield);
    let angel = put(&mut game, player, "Serra Angel", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let small = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    assert!(
        game.object_view(jackdaw)
            .keywords
            .contains(Keywords::FLYING)
    );
    cast(&mut game, murder, Some(Target::Card(angel)));
    until(&mut game, target(giant));
    reject(&mut game, target(angel));
    act(&mut game, target(giant));
    drain(&mut game, None);
    assert_eq!(game.object_view(giant).zone, Zone::Battlefield);
    let sweep = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, sweep, None);
    until(&mut game, target(small));
    reject(&mut game, target(giant));
    act(&mut game, target(small));
    drain(&mut game, None);
    assert_eq!(game.object_view(small).zone, Zone::Battlefield);
}

#[test]
fn dour_port_mage_bounces_another_creature_and_excludes_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mage = put(&mut game, player, "Dour Port-Mage", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.move_object(sacrifice, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: mage,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, target(mage));
    act(&mut game, target(bear));
    pay(&mut game);
    drain(&mut game, None);
    assert!(game.object_view(mage).tapped);
    assert_eq!(game.object_view(bear).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn gossips_talent_progresses_from_surveil_to_evasion_and_untargeted_blink() {
    let player = Player::P0;
    let mut game = scenario(player);
    let class = put(&mut game, player, "Gossip's Talent", Zone::Hand);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let top = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, class, None);
    drain(&mut game, None);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, creature, None);
    let surveil = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Graveyard,
    };
    until(&mut game, surveil.clone());
    act(&mut game, surveil);
    drain(&mut game, None);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    activate_untargeted(&mut game, class, 0);
    drain(&mut game, None);
    activate_untargeted(&mut game, class, 1);
    drain(&mut game, None);

    let attack = Action::Attack { card: attacker };
    until(&mut game, attack.clone());
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    until(&mut game, target(attacker));
    reject(&mut game, target(creature));
    act(&mut game, target(attacker));
    drain(&mut game, None);
    assert!(
        game.object_view(attacker)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    until(&mut game, Action::FinishBlockers);
    act(&mut game, Action::FinishBlockers);
    let original = game.object_ref(attacker);
    let blink = Action::ChooseOptional { accept: true };
    until(&mut game, blink.clone());
    act(&mut game, blink);
    assert_eq!(drain(&mut game, None), 1);
    assert_ne!(game.object_ref(attacker), original);
}

#[test]
fn stormchasers_talent_levels_and_plays_in_freeform_under_its_standard_ban() {
    let player = Player::P1;
    let mut game = scenario(player);
    let class = put(&mut game, player, "Stormchaser's Talent", Zone::Hand);
    let definition = game.object_view(class).card;
    let mut deck = vec![Card::Forest; 59];
    deck.push(definition);
    let config = |deck_rules| Config {
        decks: [deck.clone(), vec![Card::Forest; 60]],
        seed: 27,
        starting_player: player,
        deck_rules,
    };
    assert!(Game::new(config(DeckRules::Standard)).is_err());
    assert!(Game::new(config(DeckRules::Freeform)).is_ok());
    let spell = put(&mut game, player, "Divination", Zone::Graveyard);
    cast(&mut game, class, None);
    drain(&mut game, None);
    let original = tokens(&game, player, "Otter")[0];
    assert_eq!(game.object_view(original).colors, 10);
    assert_eq!(stats(&game, original), (1, 1));
    activate_untargeted(&mut game, class, 0);
    until(&mut game, target(spell));
    act(&mut game, target(spell));
    drain(&mut game, None);
    assert_eq!(game.object_view(spell).zone, Zone::Hand);
    activate_untargeted(&mut game, class, 1);
    drain(&mut game, None);
    cast(&mut game, spell, None);
    drain(&mut game, None);
    assert_eq!(tokens(&game, player, "Otter").len(), 2);
    assert_eq!(stats(&game, original), (2, 2));
}
