//! Exact graveyard cards, costs, turn history, and last-known power.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn select(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

#[test]
fn soulcoil_viper_pays_tap_sacrifice_and_returns_only_its_exact_target_with_finality() {
    let player = Player::P0;
    let mut game = scenario(player);
    let viper = candidate(&mut game, player, "Soulcoil Viper", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let instant = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    let mut early = game.clone();
    early.state.step = Step::Upkeep;
    reject(&mut early, activate(viper));
    let mut tapped = game.clone();
    tapped.state.objects[viper as usize].tapped = true;
    reject(&mut tapped, activate(viper));

    act(&mut game, activate(viper));
    for invalid in [viper, instant, foreign] {
        reject(&mut game, select(invalid));
    }
    act(&mut game, select(creature));
    finish_targets(&mut game);
    pay(&mut game);
    assert_eq!(game.object_view(viper).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let mut stale = game.clone();
    stale.move_object(creature, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Exile);

    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(creature, CounterKind::Finality), 1);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
}

#[test]
fn uchbenbak_counts_itself_among_eight_permanent_cards_and_returns_with_finality() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = candidate(
        &mut game,
        player,
        "Uchbenbak, the Great Mistake",
        Zone::Graveyard,
    );
    for _ in 0..6 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player.other(), "Forest", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 4];
    reject(&mut game, activate(source));
    put(&mut game, player, "Short Bow", Zone::Graveyard);
    let mut upkeep = game.clone();
    upkeep.state.step = Step::Upkeep;
    reject(&mut upkeep, activate(source));
    act(&mut game, activate(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(stats(&game, source), (6, 4));
    assert!(!game.object_view(source).tapped);
    assert!(
        game.keywords(source)
            .contains(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
    assert_eq!(game.counter_count(source, CounterKind::Finality), 1);
}

#[test]
fn stalactite_stalker_uses_turn_history_and_its_power_at_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let stalker = candidate(&mut game, player, "Stalactite Stalker", Zone::Battlefield);
    assert_eq!(stats(&game, stalker), (1, 1));
    assert!(game.keywords(stalker).contains(Keywords::MENACE));
    let mut absent = game.clone();
    end_step(&mut absent);
    settle(&mut absent);
    assert_eq!(absent.counter_count(stalker, CounterKind::PlusOne), 0);
    let hand = put(&mut game, player, "Forest", Zone::Hand);
    game.move_object(hand, Zone::Graveyard);
    game.move_object(hand, Zone::Exile);
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(stalker, CounterKind::PlusOne), 1);

    let target = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    game.add_counter(stalker, CounterKind::PlusOne, 2);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
    act(&mut game, activate(stalker));
    act(&mut game, select(target));
    finish_targets(&mut game);
    pay(&mut game);
    assert_eq!(game.object_view(stalker).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(stats(&game, target), (6, 6));
}

#[test]
fn terror_tide_counts_only_own_permanent_cards_at_resolution() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Gingerbrute", Zone::Graveyard);
    put(&mut game, player, "Forest", Zone::Graveyard);
    let removed = put(&mut game, player, "Short Bow", Zone::Graveyard);
    put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    let a = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let b = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = candidate(&mut game, player, "Terror Tide", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 2];
    cast(&mut game, spell, None);
    game.move_object(removed, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, a), (1, 1));
    assert_eq!(stats(&game, b), (1, 1));
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, a), (3, 3));
    assert_eq!(stats(&game, b), (3, 3));
}

#[test]
fn squirming_emergence_checks_the_live_nonland_permanent_threshold_twice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let wrong = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Healer's Hawk", Zone::Graveyard);
    let spell = candidate(&mut game, player, "Squirming Emergence", Zone::Hand);
    let payment = [0, 0, 1, 0, 1, 1];
    game.state.players[player.index()].mana = payment;
    let direct = Action::Cast {
        card: spell,
        target: Some(Target::Card(target)),
        payment,
    };
    reject(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, direct.clone());
    let material = candidate(&mut game, player, "@fixture/siege-front", Zone::Graveyard);
    let mut with_land = game.clone();
    let land = put(&mut with_land, player, "Forest", Zone::Graveyard);
    act(&mut with_land, Action::AnnounceCast { card: spell });
    reject(&mut with_land, select(land));

    let mut directly = game.clone();
    act(&mut directly, direct);
    settle(&mut directly);
    assert_eq!(directly.object_view(target).zone, Zone::Battlefield);

    act(&mut game, Action::AnnounceCast { card: spell });
    for invalid in [wrong, foreign] {
        reject(&mut game, select(invalid));
    }
    act(&mut game, select(target));
    finish_targets(&mut game);
    pay(&mut game);
    let mut shrank = game.clone();
    shrank.move_object(material, Zone::Exile);
    settle(&mut shrank);
    assert_eq!(shrank.object_view(target).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    assert_eq!(game.object_view(material).zone, Zone::Graveyard);
}

#[test]
fn zoyowa_only_damages_the_opponent_who_does_not_discard_or_sacrifice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = candidate(&mut game, player, "Zoyowa Lava-Tongue", Zone::Battlefield);
    assert_eq!(stats(&game, source), (2, 2));
    assert!(game.keywords(source).contains(Keywords::DEATHTOUCH));
    let hand = put(&mut game, opponent, "Forest", Zone::Hand);
    let permanent = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let turn_entry = put(&mut game, player, "Short Bow", Zone::Hand);
    let mut absent = game.clone();
    end_step(&mut absent);
    settle(&mut absent);
    assert_eq!(absent.state.players[opponent.index()].life, STARTING_LIFE);

    game.move_object(turn_entry, Zone::Graveyard);
    end_step(&mut game);
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    let object = game.object_ref(permanent);
    for (action, lost, hand_zone, permanent_zone) in [
        (
            Action::Discard { card: hand },
            0,
            Zone::Graveyard,
            Zone::Battlefield,
        ),
        (Action::Sacrifice { object }, 0, Zone::Hand, Zone::Graveyard),
        (
            Action::DeclineEffectPayment,
            3,
            Zone::Hand,
            Zone::Battlefield,
        ),
    ] {
        let mut branch = game.clone();
        act(&mut branch, action);
        settle(&mut branch);
        assert_eq!(
            branch.state.players[opponent.index()].life,
            STARTING_LIFE - lost
        );
        assert_eq!(branch.object_view(hand).zone, hand_zone);
        assert_eq!(branch.object_view(permanent).zone, permanent_zone);
    }
}

#[test]
fn mycotyrant_counts_each_fungus_or_saproling_once_and_creates_tokens_for_actual_descents() {
    let player = Player::P1;
    let mut game = scenario(player);
    let changeling = put(&mut game, player, "Changeling Wayfinder", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Changeling Wayfinder",
        Zone::Battlefield,
    );
    let source = candidate(&mut game, player, "The Mycotyrant", Zone::Hand);
    assert!(game.has_subtype(changeling, "Fungus") && game.has_subtype(changeling, "Saproling"));
    assert_eq!(stats(&game, source), (1, 1));
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (2, 2));
    assert!(game.keywords(source).contains(Keywords::TRAMPLE));

    for name in ["Short Bow", "Forest", "Opt"] {
        let card = put(&mut game, player, name, Zone::Hand);
        game.move_object(card, Zone::Graveyard);
        game.move_object(card, Zone::Exile);
    }
    end_step(&mut game);
    settle(&mut game);
    let tokens: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.is_token())
        .collect();
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        assert_eq!((token.power, token.toughness, token.colors), (1, 1, 4));
        assert!(token.has_subtype("Fungus"));
        assert!(token.keywords.contains(Keywords::CANT_BLOCK));
    }
    assert_eq!(stats(&game, source), (4, 4));
}
