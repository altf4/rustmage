//! Combat, entry, and triggered choices on seven Bloomburrow creatures.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("expected resolution choice in {:?}", game.state.pending);
}

fn idle(game: &mut Game) {
    advance(game, |g| {
        g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending == Pending::Priority
    });
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

fn choose_target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn target_choice(game: &mut Game) {
    advance(game, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
    });
}

#[test]
fn bria_grants_an_independent_prowess_and_chooses_a_temporary_unblockable_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bria = put(&mut game, player, "Bria, Riptide Rogue", Zone::Battlefield);
    let dowser = put(&mut game, player, "Shipwreck Dowser", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    target_choice(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    choose_target(&mut game, dowser);
    idle(&mut game);
    assert_eq!(stats(&game, bria), (4, 4));
    assert_eq!(stats(&game, dowser), (5, 5));
    assert_eq!(stats(&game, opponent), (2, 2));
    assert!(game.keywords(dowser).contains(Keywords::CANT_BE_BLOCKED));
    assert!(!game.keywords(bria).contains(Keywords::CANT_BE_BLOCKED));
    game.finish_cleanup();
    assert_eq!(stats(&game, dowser), (3, 3));
    assert!(!game.keywords(dowser).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn byrke_targets_up_to_two_creatures_then_doubles_a_declared_attackers_counter() {
    let player = Player::P1;
    let mut game = scenario(player);
    let byrke = put(&mut game, player, "Byrke, Long Ear of the Law", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, byrke, None);
    target_choice(&mut game);
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    idle(&mut zero);
    assert_eq!(zero.counter_count(own, CounterKind::PlusOne), 0);

    choose_target(&mut game, own);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    choose_target(&mut game, opposing);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    idle(&mut game);
    assert!(game.keywords(byrke).contains(Keywords::VIGILANCE));
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);
    attack(&mut game, &[own]);
    idle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);
}

#[test]
fn finneas_counts_other_rabbits_and_creature_tokens_once_then_checks_live_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let finneas = put(&mut game, player, "Finneas, Ace Archer", Zone::Battlefield);
    let rabbit = put(
        &mut game,
        player,
        "Head of the Homestead",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let rabbit_token = game.state.objects.len() as CardId;
    game.create_tokens(player, Card(5285), 1, false);
    let fish = game.state.objects.len() as CardId;
    game.create_tokens(player, Card(5317), 1, false);
    attack(&mut game, &[finneas]);
    idle(&mut game);
    for recipient in [rabbit, rabbit_token, fish] {
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    }
    for excluded in [finneas, bear] {
        assert_eq!(game.counter_count(excluded, CounterKind::PlusOne), 0);
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert!(
        game.keywords(finneas)
            .contains(Keywords::REACH.union(Keywords::VIGILANCE))
    );
    assert!(!game.object_view(finneas).tapped);
}

#[test]
fn kastral_groups_bird_damage_and_supports_each_mode() {
    let player = Player::P1;
    let mut game = scenario(player);
    let kastral = put(
        &mut game,
        player,
        "Kastral, the Windcrested",
        Zone::Battlefield,
    );
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let kindred = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    let hand = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let graveyard = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let damage = [kastral, hawk].map(|id| DamageEvent {
        preventable: true,
        source: game.damage_source(id),
        target: Target::Player(player.other()),
        amount: 1,
        combat: true,
    });
    game.apply_damage_batch(&damage);
    game.advance_replacements(false);
    game.prepare_triggers();
    assert!(matches!(game.state.pending, Pending::Modes { .. }));
    assert!(game.state.ordering_triggers.is_empty());
    assert!(game.keywords(kastral).contains(Keywords::FLYING));

    for card in [hand, graveyard] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseMode { mode: 0 });
        assert_eq!(branch.state.stack.len(), 1);
        advance(&mut branch, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let public = branch.observe(player.other()).effect_choice.unwrap();
        assert_eq!(public.candidates, [branch.object_ref(graveyard)]);
        let object = branch.object_ref(card);
        act(&mut branch, Action::ChooseEffectObject { object });
        assert_eq!(branch.object_view(card).zone, Zone::Battlefield);
        assert_eq!(branch.counter_count(card, CounterKind::Finality), 1);
    }

    let mut counters = game.clone();
    act(&mut counters, Action::ChooseMode { mode: 1 });
    idle(&mut counters);
    for bird in [kastral, hawk, kindred] {
        assert_eq!(counters.counter_count(bird, CounterKind::PlusOne), 1);
    }

    act(&mut game, Action::ChooseMode { mode: 2 });
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn plumecreed_triggers_for_own_flyers_and_only_targets_a_controlled_nonflyer() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Plumecreed Mentor", Zone::Hand);
    let ground = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    cast(&mut game, mentor, None);
    target_choice(&mut game);
    assert!(game.keywords(mentor).contains(Keywords::FLYING));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(mentor),
        },
    );
    choose_target(&mut game, ground);
    idle(&mut game);
    assert_eq!(game.counter_count(ground, CounterKind::PlusOne), 1);

    let plain = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, plain, None);
    idle(&mut game);
    assert_eq!(game.counter_count(ground, CounterKind::PlusOne), 1);
    cast(&mut game, hawk, None);
    target_choice(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(hawk),
        },
    );
    choose_target(&mut game, ground);
    idle(&mut game);
    assert_eq!(game.counter_count(ground, CounterKind::PlusOne), 2);
}

#[test]
fn seedpod_can_use_either_hybrid_color_and_pumps_a_ground_creature_when_it_attacks() {
    let player = Player::P1;
    for color in [0, 1] {
        let mut game = scenario(player);
        let squire = put(&mut game, player, "Seedpod Squire", Zone::Hand);
        let ground = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        game.state.players[player.index()].mana[color] = 1;
        cast(&mut game, squire, None);
        idle(&mut game);
        assert!(game.keywords(squire).contains(Keywords::FLYING));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        game.state.objects[squire as usize].summoning_sick = false;
        attack(&mut game, &[squire]);
        target_choice(&mut game);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(squire),
            },
        );
        choose_target(&mut game, ground);
        idle(&mut game);
        assert_eq!(stats(&game, ground), (3, 3));
        game.finish_cleanup();
        assert_eq!(stats(&game, ground), (2, 2));
    }
}

#[test]
fn zoraline_pays_before_choosing_a_permanent_and_rewards_each_attacking_bat() {
    let player = Player::P0;
    let mut game = scenario(player);
    let zoraline = put(&mut game, player, "Zoraline, Cosmos Caller", Zone::Hand);
    let equipment = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    cast(&mut game, zoraline, None);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    assert!(
        game.keywords(zoraline)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    assert_eq!(game.state.players[player.index()].life, 20);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [1, 0, 1, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].life, 18);
    target_choice(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    choose_target(&mut game, equipment);
    idle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(equipment, CounterKind::Finality), 1);

    put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let bat = game.state.objects.len() as CardId;
    game.create_tokens(player, Card(5276), 1, false);
    for id in [zoraline, bat] {
        game.state.objects[id as usize].summoning_sick = false;
    }
    attack(&mut game, &[zoraline, bat]);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    act(&mut game, Action::DeclineResolutionMana);
    idle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert!(!game.object_view(zoraline).tapped);
}
