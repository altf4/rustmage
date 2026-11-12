use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn disguise(game: &mut Game, card: CardId) -> ObjectRef {
    act(game, Action::AnnounceDisguise { card });
    pay(game);
    settle(game);
    game.object_ref(card)
}

fn turn_up(game: &mut Game, object: ObjectRef) {
    choose(
        game,
        |action| matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object),
    );
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
}

fn cloak(game: &mut Game, player: Player, name: &str) -> ObjectRef {
    let card = put(game, player, name, Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let object = game.object_ref(card);
    assert!(game.is_face_down(object));
    object
}

fn pass_to(game: &mut Game, pending: Pending) {
    for _ in 0..12 {
        if game.state.pending == pending {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("missing combat choice");
}

fn order_triggers(game: &mut Game) {
    for _ in 0..8 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("trigger order did not finish");
}

fn tokens(game: &Game, controller: Player, card: Card) -> Vec<CardId> {
    game.controlled_token_ids(controller)
        .filter(|&id| game.object_view(id).card == card)
        .collect()
}

#[test]
fn assemble_uses_the_proposed_face_and_tracks_each_exact_provider() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Assemble the Players", Zone::Hand);
    cast(&mut game, first, None);
    settle(&mut game);
    let second = put(&mut game, player, "Assemble the Players", Zone::Battlefield);
    let third = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let next = put(&mut game, player, "Savannah Lions", Zone::Library);
    let hidden = put(&mut game, player, "Defenestrated Phantom", Zone::Library);
    let top = game.object_ref(hidden);
    let visible = |game: &Game, viewer| {
        game.observe(viewer).known_library_top[player.index()]
            .as_ref()
            .map(|top| top.object)
    };
    let cast_from_top = |game: &Game, id| {
        let object = game.object_ref(id);
        game.legal_actions()
            .into_iter()
            .filter(|action| {
                matches!(action, Action::AnnounceLibraryTop {
            object: candidate, ..
        } if *candidate == object)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(visible(&game, player), Some(top));
    assert_eq!(visible(&game, opponent), None);
    let actions = cast_from_top(&game, hidden);
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().all(|action| matches!(
        action,
        Action::AnnounceLibraryTop {
            method: CastMethod::Disguise,
            ..
        }
    )));
    let announce = actions
        .into_iter()
        .find(|action| {
            matches!(action, Action::AnnounceLibraryTop {
        permission, ..
    } if permission.source == game.object_ref(first))
        })
        .unwrap();
    let saved = game.snapshot();
    act(&mut game, announce);
    assert_eq!(visible(&game, player), None);
    assert_eq!(
        game.observe(player).face_down_spells[0].underlying,
        Card::from_name("Defenestrated Phantom")
    );
    assert_eq!(game.observe(opponent).face_down_spells[0].underlying, None);

    let mut restored = game.clone();
    restored.restore(&saved).unwrap();
    assert_eq!(cast_from_top(&restored, hidden).len(), 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(visible(&game, player), Some(game.object_ref(next)));
    assert_eq!(stats(&game, hidden), (2, 2));
    assert_eq!(
        game.observe(opponent).face_down_permanents[0].underlying,
        None
    );
    let object = game.object_ref(hidden);
    turn_up(&mut game, object);
    assert_eq!(stats(&game, hidden), (4, 3));
    assert!(game.object_view(hidden).keywords.contains(Keywords::FLYING));

    let [remaining]: [Action; 1] = cast_from_top(&game, next).try_into().unwrap();
    assert!(matches!(&remaining, Action::AnnounceLibraryTop {
        permission, method: CastMethod::Normal, ..
    } if permission.source == game.object_ref(second)));
    act(&mut game, remaining);
    pay(&mut game);
    settle(&mut game);
    assert!(cast_from_top(&game, third).is_empty());
}

#[test]
fn vindicators_paid_x_combines_zones_and_links_return_to_the_exact_source() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let fallen = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let third = put(&mut game, opponent, "Healer's Hawk", Zone::Graveyard);
    let angel = put(&mut game, player, "Aurelia's Vindicator", Zone::Hand);
    let object = disguise(&mut game, angel);
    choose(
        &mut game,
        |action| matches!(action, Action::BeginFaceUpX { object: candidate, .. } if *candidate == object),
    );
    act(&mut game, Action::ChooseX { value: 2 });
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    reject(&mut game, target(angel));
    act(&mut game, target(victim));
    act(&mut game, target(fallen));
    reject(&mut game, target(third));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    assert_eq!(game.state.stack.last().unwrap().x, Some(2));
    settle(&mut game);
    assert_eq!(stats(&game, angel), (4, 2));
    assert!(game.object_view(angel).keywords.contains(Keywords::FLYING));
    assert!(
        game.object_view(angel)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.object_view(fallen).zone, Zone::Exile);
    assert_eq!(game.object_view(third).zone, Zone::Graveyard);

    act(&mut game, Action::Pass);
    let enemy_bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    cast(&mut game, enemy_bounce, Some(Target::Card(angel)));
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(angel).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    let own_bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, own_bounce, Some(Target::Card(angel)));
    settle(&mut game);
    assert_eq!(game.object_view(angel).zone, Zone::Hand);
    assert_eq!(game.object_view(victim).zone, Zone::Hand);
    assert_eq!(game.object_view(fallen).zone, Zone::Hand);
    assert_eq!(game.object_view(third).zone, Zone::Graveyard);
}

#[test]
fn essence_untaps_its_creatures_and_grants_hexproof_in_response() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Essence of Antiquity", Zone::Hand);
    let source = disguise(&mut game, card);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let cub = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    for creature in [bear, cub, enemy] {
        game.state.objects[creature as usize].tapped = true;
    }
    game.add_counter(cub, CounterKind::Stun, 1);
    act(&mut game, Action::Pass);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(bear)));
    act(&mut game, Action::Pass);
    turn_up(&mut game, source);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
    assert!(!game.object_view(bear).tapped);
    assert!(game.object_view(bear).keywords.contains(Keywords::HEXPROOF));
    assert!(game.object_view(cub).tapped);
    assert_eq!(game.counter_count(cub, CounterKind::Stun), 0);
    assert!(game.object_view(enemy).tapped);
    assert!(
        !game
            .object_view(enemy)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    assert_eq!(stats(&game, card), (1, 10));
}

#[test]
fn forum_familiar_bounces_the_controlled_land_to_its_owner_and_rechecks_the_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let cat = put(&mut game, player, "Forum Familiar", Zone::Hand);
    let stolen = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    let object = disguise(&mut game, cat);
    turn_up(&mut game, object);
    reject(&mut game, target(cat));
    reject(&mut game, target(enemy));
    act(&mut game, target(stolen));
    let mut stale = game.clone();
    stale.move_object(stolen, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stats(&stale, cat), (1, 1));
    settle(&mut game);
    assert_eq!(stats(&game, cat), (2, 2));
    assert_eq!(game.object_view(stolen).owner, opponent);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
}

#[test]
fn karlov_watchdog_bars_opposing_turn_up_on_its_turn_and_rewards_three_attackers() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let hidden = put(&mut game, opponent, "Bubble Smuggler", Zone::Hand);
    let object = disguise(&mut game, hidden);
    let watchdog = put(&mut game, player, "Karlov Watchdog", Zone::Battlefield);
    let has_turn_up = |game: &Game| {
        game.legal_actions().iter().any(|action| {
            matches!(action, Action::BeginFaceUp {
        object: candidate, ..
    } if *candidate == object)
        })
    };
    assert!(has_turn_up(&game));
    game.state.active = player;
    assert!(!has_turn_up(&game));
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let third = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    game.state.priority = player;
    game.state.pending = Pending::Attackers;
    game.state.step = Step::Attackers;
    for card in [first, second, third] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(stats(&game, watchdog), (4, 3));
    assert_eq!(stats(&game, first), (3, 3));
    assert_eq!(stats(&game, hidden), (2, 2));
    assert!(
        game.object_view(watchdog)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
}

#[test]
fn museum_nightwatch_death_creates_a_detective_only_for_the_face_up_copy() {
    let player = Player::P0;
    let mut game = scenario(player);
    let down = put(&mut game, player, "Museum Nightwatch", Zone::Hand);
    let up = put(&mut game, player, "Museum Nightwatch", Zone::Hand);
    disguise(&mut game, down);
    let object = disguise(&mut game, up);
    turn_up(&mut game, object);
    let wrath = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    settle(&mut game);
    assert_eq!(game.object_view(down).zone, Zone::Graveyard);
    assert_eq!(game.object_view(up).zone, Zone::Graveyard);
    let detectives = tokens(&game, player, Card::from_catalogue_id(5329));
    assert_eq!(detectives.len(), 1);
    let detective = game.object_view(detectives[0]);
    assert_eq!(stats(&game, detective.id), (2, 2));
    assert_eq!(detective.colors, 3);
    assert!(detective.has_subtype("Detective"));
}

#[test]
fn gatekeeper_uses_resolution_control_to_choose_tapped_return_or_detective() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let gate = put(&mut game, player, "Unyielding Gatekeeper", Zone::Hand);
    let borrowed = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.record_control(game.object_ref(borrowed), player, None);
    let gate = disguise(&mut game, gate);
    turn_up(&mut game, gate);
    reject(&mut game, target(gate.card));
    reject(&mut game, target(land));
    act(&mut game, target(borrowed));
    let old = game.object_ref(borrowed);
    let mut enemy = game.clone();
    enemy.record_control(old, opponent, None);
    settle(&mut enemy);
    assert_eq!(enemy.object_view(borrowed).zone, Zone::Exile);
    assert_eq!(
        tokens(&enemy, opponent, Card::from_catalogue_id(5329)).len(),
        1
    );
    assert!(tokens(&enemy, player, Card::from_catalogue_id(5329)).is_empty());
    settle(&mut game);
    let returned = game.object_view(borrowed);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, player);
    assert_eq!(returned.owner, opponent);
    assert!(returned.tapped);
    assert_ne!(returned.reference, old);
}

#[test]
fn coveted_falcon_gives_away_actual_targets_then_reclaims_an_owned_permanent_on_attack() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let falcon = put(&mut game, player, "Coveted Falcon", Zone::Hand);
    let artifact = put(&mut game, player, "Candlestick", Zone::Battlefield);
    let stale = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let object = disguise(&mut game, falcon);
    turn_up(&mut game, object);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    act(&mut game, target(artifact));
    act(&mut game, target(stale));
    act(&mut game, Action::FinishTargets);
    game.move_object(stale, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).controller, opponent);
    assert_eq!(game.hand_count(player), 1);

    game.state.objects[falcon as usize].summoning_sick = false;
    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: falcon });
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(enemy));
    act(&mut game, target(artifact));
    settle(&mut game);
    assert_eq!(game.object_view(artifact).controller, player);
}

#[test]
fn cryptic_coat_cloaks_and_attaches_then_leaves_the_creature_in_place_when_bounced() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let coat = put(&mut game, player, "Cryptic Coat", Zone::Hand);
    let hidden = put(&mut game, player, "Bubble Smuggler", Zone::Library);
    cast(&mut game, coat, None);
    settle(&mut game);
    let object = game.object_ref(hidden);
    assert!(game.is_face_down(object));
    let public = game.observe(opponent).face_down_permanents;
    assert_eq!(public[0].cause, FaceDownCause::Cloak);
    assert_eq!(public[0].underlying, None);
    assert_eq!(game.object_view(coat).attached_to, Some(object));
    assert_eq!(stats(&game, hidden), (3, 2));
    assert!(
        game.object_view(hidden)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    // The printed mana route is cheaper than this creature's disguise cost.
    game.state.players[player.index()].mana = [0, 3, 0, 0, 0, 1];
    turn_up(&mut game, object);
    assert_eq!(game.object_ref(hidden), object);
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 4);
    assert_eq!(stats(&game, hidden), (7, 5));

    choose(&mut game, |action| {
        matches!(action,
        Action::Activate { card, ability: 0, .. }
        | Action::AnnounceActivate { card, ability: 0, .. }
        if *card == coat)
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(coat).zone, Zone::Hand);
    assert_eq!(game.object_ref(hidden), object);
    assert_eq!(stats(&game, hidden), (6, 5));
    assert!(
        !game
            .object_view(hidden)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
}

#[test]
fn exit_specialist_bounces_another_creature_and_uses_live_blocker_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let specialist = put(&mut game, player, "Exit Specialist", Zone::Hand);
    let bounced = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let large = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let object = disguise(&mut game, specialist);
    turn_up(&mut game, object);
    reject(&mut game, target(specialist));
    act(&mut game, target(bounced));
    settle(&mut game);
    assert_eq!(game.object_view(bounced).zone, Zone::Hand);
    game.state.objects[specialist as usize].summoning_sick = false;
    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: specialist });
    act(&mut game, Action::FinishAttackers);
    pass_to(&mut game, Pending::Blockers);
    let legal = Action::Block {
        blocker: small,
        attacker: specialist,
    };
    let illegal = Action::Block {
        blocker: large,
        attacker: specialist,
    };
    assert!(game.legal_actions().contains(&legal));
    reject(&mut game, illegal);
    game.add_counter(small, CounterKind::PlusOne, 1);
    reject(&mut game, legal);
}

#[test]
fn mistway_spys_delayed_clues_outlive_the_spy_and_count_each_combat_source() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spy = put(&mut game, player, "Mistway Spy", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let object = disguise(&mut game, spy);
    turn_up(&mut game, object);
    settle(&mut game);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(spy)));
    settle(&mut game);
    assert_eq!(game.object_view(spy).zone, Zone::Hand);
    pass_to(&mut game, Pending::Attackers);
    for card in [first, second] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_to(&mut game, Pending::Blockers);
    act(&mut game, Action::FinishBlockers);
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 16);
    assert_eq!(
        tokens(&game, player, Card::from_catalogue_id(5296)).len(),
        2
    );
}

#[test]
fn detective_turn_up_observers_distinguish_self_entry_and_optional_looting() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enforcer = put(&mut game, player, "Perimeter Enforcer", Zone::Hand);
    cast(&mut game, enforcer, None);
    settle(&mut game);
    assert_eq!(stats(&game, enforcer), (1, 1));
    let proof = put(
        &mut game,
        player,
        "Case of the Pilfered Proof",
        Zone::Battlefield,
    );
    let detective = cloak(&mut game, player, "Perimeter Enforcer");
    turn_up(&mut game, detective);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, enforcer), (2, 2));
    assert_eq!(stats(&game, detective.card), (3, 3));
    assert_eq!(game.counter_count(detective.card, CounterKind::PlusOne), 1);
    game.move_object(enforcer, Zone::Graveyard);
    game.move_object(detective.card, Zone::Graveyard);
    game.move_object(proof, Zone::Graveyard);
    let inspector = put(&mut game, player, "Projektor Inspector", Zone::Hand);
    let discard = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, inspector, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.hand_count(player), 1);
    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 1);
}
