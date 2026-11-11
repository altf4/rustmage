//! Permanent interaction and layered rules on seven Bloomburrow creatures.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];

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

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn tap_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn announce(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn baylen_taps_distinct_tokens_for_mana_draw_and_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let baylen = put(&mut game, player, "Baylen, the Haymaker", Zone::Battlefield);
    game.state.objects[baylen as usize].summoning_sick = true;
    let physical = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = game.state.objects.len() as CardId;
    game.create_tokens(player.other(), Card::from_catalogue_id(5304), 1, false);
    let start = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::from_catalogue_id(5317), 2, false);
    game.create_tokens(player, Card::from_catalogue_id(5304), 2, false);
    game.state.players[player.index()].mana = [0; 6];

    let mut mana = game.clone();
    choose(&mut mana, |action| {
        matches!(action, Action::AnnounceMana { card, production, .. }
            if *card == baylen && [GREEN, [0; 6]].contains(production))
    });
    for card in [physical, enemy] {
        let action = tap_cost(&mana, card);
        reject(&mut mana, action);
    }
    let first = tap_cost(&mana, start);
    act(&mut mana, first.clone());
    reject(&mut mana, first);
    let second = tap_cost(&mana, start + 2);
    act(&mut mana, second);
    pay(&mut mana);
    if matches!(mana.state.pending, Pending::EffectMana { .. }) {
        act(&mut mana, Action::ChooseMana { production: GREEN });
    }
    assert!(mana.state.stack.is_empty());
    assert_eq!(mana.state.players[player.index()].mana, GREEN);
    assert!(!mana.object_view(baylen).tapped);
    assert!(mana.object_view(start).tapped && mana.object_view(start + 2).tapped);

    for (ability, count) in [(1, 3), (2, 4)] {
        let mut branch = game.clone();
        announce(&mut branch, baylen, ability);
        for token in start..start + count {
            let action = tap_cost(&branch, token);
            act(&mut branch, action);
        }
        pay(&mut branch);
        idle(&mut branch);
        assert!(!branch.object_view(baylen).tapped);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            if ability == 1 { 1 } else { 0 }
        );
        assert_eq!(
            branch.counter_count(baylen, CounterKind::PlusOne),
            if ability == 2 { 3 } else { 0 }
        );
        assert_eq!(
            branch.keywords(baylen).contains(Keywords::TRAMPLE),
            ability == 2
        );
        for token in start..start + 4 {
            assert_eq!(branch.object_view(token).tapped, token < start + count);
        }
    }
}

#[test]
fn clement_returns_a_smaller_creature_and_gives_frogs_restricted_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    let clement = put(&mut game, player, "Clement, the Worrywort", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let frog = put(&mut game, player, "Pond Prophet", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Pond Prophet", Zone::Battlefield);
    cast(&mut game, clement, None);
    advance(&mut game, |g| g.legal_actions().contains(&target(bear)));
    reject(&mut game, target(large));
    reject(&mut game, target(enemy));
    reject(&mut game, target(clement));
    act(&mut game, target(bear));
    idle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Hand);
    assert!(game.keywords(clement).contains(Keywords::VIGILANCE));

    game.state.players[player.index()].mana = [0; 6];
    let grantor = game.object_ref(clement);
    reject(
        &mut game,
        Action::TapGrantedMana {
            card: clement,
            grantor,
            production: GREEN,
        },
    );
    reject(
        &mut game,
        Action::TapGrantedMana {
            card: enemy,
            grantor,
            production: GREEN,
        },
    );
    let mut blue = game.clone();
    blue.state.objects[clement as usize].summoning_sick = false;
    act(
        &mut blue,
        Action::TapGrantedMana {
            card: clement,
            grantor,
            production: BLUE,
        },
    );
    assert_eq!(blue.state.players[player.index()].mana, BLUE);
    let opt = put(&mut blue, player, "Opt", Zone::Hand);
    reject(&mut blue, Action::AnnounceCast { card: opt });
    act(
        &mut game,
        Action::TapGrantedMana {
            card: frog,
            grantor,
            production: GREEN,
        },
    );
    assert_eq!(game.state.players[player.index()].mana, GREEN);
    let elves = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    cast(&mut game, elves, None);
    idle(&mut game);
    assert_eq!(game.object_view(elves).zone, Zone::Battlefield);
}

#[test]
fn gev_counts_opponents_who_lost_life_and_its_ward_uses_the_targeting_payer() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let gev = put(&mut game, player, "Gev, Scaled Scorch", Zone::Hand);
    let lizard = put(&mut game, player, "Cindering Cutthroat", Zone::Hand);
    game.lose_life(opponent, 2);
    game.gain_life(opponent, 2);
    cast(&mut game, gev, None);
    idle(&mut game);
    assert_eq!(game.counter_count(gev, CounterKind::PlusOne), 0);

    cast(&mut game, lizard, None);
    let hit = Action::ChooseTarget {
        target: Target::Player(opponent),
    };
    advance(&mut game, |g| g.legal_actions().contains(&hit));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(&mut game, hit);
    idle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    // Cutthroat also supplies one counter through its own entry replacement.
    assert_eq!(game.counter_count(lizard, CounterKind::PlusOne), 2);

    let removal = put(&mut game, opponent, "Hero's Downfall", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(gev)));
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::ResolutionLife { .. })
    });
    assert_eq!(game.actor(), Some(opponent));
    let mut paid = game.clone();
    act(&mut game, Action::DeclineResolutionLife);
    idle(&mut game);
    assert_eq!(game.object_view(gev).zone, Zone::Battlefield);
    act(&mut paid, Action::PayResolutionLife);
    idle(&mut paid);
    assert_eq!(paid.state.players[opponent.index()].life, 17);
    assert_eq!(paid.object_view(gev).zone, Zone::Graveyard);
}

#[test]
fn lilysplash_can_blink_only_another_owned_or_borrowed_creature_at_sorcery_speed() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Lilysplash Mentor", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let borrowed = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let original = game.object_ref(borrowed);
    game.record_control(original, player, None);
    game.add_counter(borrowed, CounterKind::PlusOne, 2);
    assert!(game.keywords(mentor).contains(Keywords::REACH));

    let activation = Action::AnnounceActivate {
        card: mentor,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::BeginCombat;
    reject(&mut game, activation.clone());
    game.state.step = Step::Main1;
    act(&mut game, activation);
    reject(&mut game, target(mentor));
    reject(&mut game, target(enemy));
    act(&mut game, target(borrowed));
    pay(&mut game);
    idle(&mut game);
    assert_ne!(game.object_ref(borrowed), original);
    assert_eq!(game.object_view(borrowed).zone, Zone::Battlefield);
    assert_eq!(game.object_view(borrowed).controller, player.other());
    assert_eq!(game.counter_count(borrowed, CounterKind::PlusOne), 1);
}

#[test]
fn mabel_buffs_other_mice_and_creates_the_exact_cragflame_equipment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mabel = put(&mut game, player, "Mabel, Heir to Cragflame", Zone::Hand);
    let mouse = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Three Tree Mascot",
        Zone::Battlefield,
    );
    cast(&mut game, mabel, None);
    idle(&mut game);
    assert_eq!(stats(&game, mabel), (3, 3));
    assert_eq!(stats(&game, mouse), (3, 2));
    assert_eq!(stats(&game, enemy), (2, 1));

    let equipment = game.state.objects.len() as CardId - 1;
    let view = game.object_view(equipment);
    assert_eq!(view.name, "Cragflame");
    assert!(view.is_legendary() && view.is_token() && view.has_subtype("Equipment"));
    assert_eq!(view.colors, 0);
    announce(&mut game, equipment, 0);
    act(&mut game, target(mouse));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    idle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(mouse))
    );
    assert_eq!(stats(&game, mouse), (4, 3));
    let benefits = Keywords::VIGILANCE
        .union(Keywords::TRAMPLE)
        .union(Keywords::HASTE);
    assert!(game.keywords(mouse).contains(benefits));
}

#[test]
fn starseer_triggers_once_on_a_net_zero_life_change_and_opponent_chooses_the_payment() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let mentor = put(&mut game, player, "Starseer Mentor", Zone::Battlefield);
    let discard = put(&mut game, opponent, "Forest", Zone::Hand);
    let permanent = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    game.gain_life(player, 1);
    game.lose_life(player, 1);
    assert!(
        game.keywords(mentor)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    let target_opponent = Action::ChooseTarget {
        target: Target::Player(opponent),
    };
    advance(&mut game, |g| g.legal_actions().contains(&target_opponent));
    act(&mut game, target_opponent);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    let protected = Action::Sacrifice {
        object: game.object_ref(land),
    };
    reject(&mut game, protected);

    let mut sacrifice = game.clone();
    let action = Action::Sacrifice {
        object: sacrifice.object_ref(permanent),
    };
    act(&mut sacrifice, action);
    assert_eq!(sacrifice.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(sacrifice.state.players[opponent.index()].life, 20);
    let mut discard_branch = game.clone();
    act(&mut discard_branch, Action::Discard { card: discard });
    assert_eq!(discard_branch.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(discard_branch.state.players[opponent.index()].life, 20);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.pending, Pending::Priority);
}

#[test]
fn ygra_grants_food_activation_and_ward_sacrifice_and_counts_real_graveyard_moves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ygra = put(&mut game, player, "Ygra, Eater of All", Zone::Battlefield);
    let food = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy_food = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    assert!(!game.has_subtype(ygra, "Food"));
    assert!(!game.has_subtype(plain, "Food"));
    for card in [food, enemy_food] {
        assert_eq!(game.object_view(card).kinds, 1 | 16);
        assert!(game.has_subtype(card, "Food") && game.has_subtype(card, "Bear"));
    }

    let activation = Action::AnnounceArtifactFood {
        source: game.object_ref(food),
        grantor: game.object_ref(ygra),
        ability: 0,
    };
    act(&mut game, activation);
    pay(&mut game);
    idle(&mut game);
    assert_eq!(game.object_view(food).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.counter_count(ygra, CounterKind::PlusOne), 2);

    let removal = put(&mut game, opponent, "Hero's Downfall", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(ygra)));
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let bad = Action::Sacrifice {
        object: game.object_ref(plain),
    };
    reject(&mut game, bad);

    let mut paid = game.clone();
    let payment = Action::Sacrifice {
        object: paid.object_ref(enemy_food),
    };
    act(&mut paid, payment);
    advance(&mut paid, |g| {
        g.counter_count(ygra, CounterKind::PlusOne) == 4
    });
    idle(&mut paid);
    assert_eq!(paid.object_view(ygra).zone, Zone::Graveyard);
    assert_eq!(paid.object_view(enemy_food).zone, Zone::Graveyard);

    act(&mut game, Action::DeclineEffectPayment);
    idle(&mut game);
    assert_eq!(game.object_view(ygra).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy_food).zone, Zone::Battlefield);
    game.add_counter(enemy_food, CounterKind::Finality, 1);
    game.move_object(enemy_food, Zone::Graveyard);
    idle(&mut game);
    assert_eq!(game.object_view(enemy_food).zone, Zone::Exile);
    assert_eq!(game.counter_count(ygra, CounterKind::PlusOne), 2);
}
