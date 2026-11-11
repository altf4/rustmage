//! Real declaration, tap, damage and artifact-trigger consumers.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GNOME: Card = Card::from_catalogue_id(5267);
const TREASURE: Card = Card::from_catalogue_id(5302);
const MAP: Card = Card::from_catalogue_id(5284);
const MERFOLK: Card = Card::from_catalogue_id(5576);

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, cards: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.controller == player && o.card == card && o.is_token()
        })
        .collect()
}

#[test]
fn adaptive_gemguard_pays_two_otherwise_independent_taps_including_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gem = candidate(&mut game, player, "Adaptive Gemguard", Zone::Battlefield);
    let sick = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    game.state.objects[sick as usize].summoning_sick = true;
    game.state.objects[gem as usize].summoning_sick = true;
    game.state.step = Step::End;
    reject(&mut game, activate(gem));
    game.state.step = Step::Main1;
    act(&mut game, activate(gem));
    let cost = |game: &Game, id| Action::ChooseAdditionalCost {
        object: game.object_ref(id),
    };
    let invalid = cost(&game, enemy);
    reject(&mut game, invalid);
    let first = cost(&game, gem);
    act(&mut game, first.clone());
    reject(&mut game, first);
    let second = cost(&game, sick);
    act(&mut game, second);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert!(game.object_view(gem).tapped && game.object_view(sick).tapped);
    settle(&mut game);
    assert_eq!(game.counter_count(gem, CounterKind::PlusOne), 1);
}

#[test]
fn anim_pakal_triggers_once_for_a_group_and_uses_last_known_counters_if_removed() {
    let player = Player::P1;
    let mut game = scenario(player);
    let anim = candidate(
        &mut game,
        player,
        "Anim Pakal, Thousandth Moon",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let gnome = put(
        &mut game,
        player,
        "Cartographer's Companion",
        Zone::Battlefield,
    );
    game.add_counter(anim, CounterKind::PlusOne, 2);
    let mut gnome_only = game.clone();
    attack(&mut gnome_only, &[gnome]);
    assert!(gnome_only.state.stack.is_empty());
    attack(&mut game, &[first, second, gnome]);
    assert_eq!(game.state.stack.len(), 1);
    let mut departed = game.clone();
    departed.move_object(anim, Zone::Graveyard);
    settle(&mut departed);
    assert_eq!(tokens(&departed, player, GNOME).len(), 2);
    settle(&mut game);
    assert_eq!(game.counter_count(anim, CounterKind::PlusOne), 3);
    let gn = tokens(&game, player, GNOME);
    assert_eq!(gn.len(), 3);
    for gnome in gn {
        assert_eq!(gnome.kinds, 17);
        assert!(gnome.has_subtype("Gnome"));
        assert_eq!(gnome.colors, 0);
        assert!(gnome.tapped && game.state.attackers.iter().any(|a| a.card == gnome.id));
    }
    assert!(!game.state.attackers.iter().any(|a| a.card == anim));
}

#[test]
fn bedrock_tortoise_grants_own_turn_hexproof_and_uses_only_greater_toughness() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tortoise = candidate(&mut game, player, "Bedrock Tortoise", Zone::Battlefield);
    let lion = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    for id in [tortoise, lion] {
        assert!(game.object_view(id).keywords.contains(Keywords::HEXPROOF));
    }
    assert!(
        !game
            .object_view(enemy)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    let mut opposing = game.clone();
    opposing.state.active = player.other();
    assert!(
        !opposing
            .object_view(lion)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    attack(&mut game, &[tortoise, lion]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 12);
}

#[test]
fn bloodletter_doubles_opposing_life_loss_on_the_owners_turn_without_doubling_damage() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = candidate(
        &mut game,
        player,
        "Bloodletter of Aclazotz",
        Zone::Battlefield,
    );
    candidate(
        &mut game,
        player,
        "Bloodletter of Aclazotz",
        Zone::Battlefield,
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    let mut off_turn = game.clone();
    off_turn.state.active = opponent;
    cast(&mut off_turn, shock, Some(Target::Player(opponent)));
    settle(&mut off_turn);
    assert_eq!(off_turn.state.players[opponent.index()].life, 18);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 12);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, second, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 2);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
}

#[test]
fn brackish_blunder_samples_tapped_at_resolution_and_returns_to_the_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = candidate(&mut game, player, "Brackish Blunder", Zone::Hand);
    game.change_controller(game.object_ref(creature), player);
    cast(&mut game, spell, Some(Target::Card(creature)));
    let mut tapped = game.clone();
    tapped.tap_objects(&[creature]);
    settle(&mut tapped);
    assert_eq!(tapped.object_view(creature).zone, Zone::Hand);
    assert_eq!(tapped.object_view(creature).owner, player.other());
    let map = tokens(&tapped, player, MAP);
    assert_eq!(map.len(), 1);
    assert!(map[0].has_subtype("Map"));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert!(tokens(&game, player, MAP).is_empty());
}

#[test]
fn combat_pumps_use_the_declaration_fact_including_cavalrys_block_trigger() {
    let player = Player::P0;
    for (name, first, second, expected) in [
        (
            "Brazen Blademaster",
            "Candy Trail",
            Some("Short Bow"),
            (4, 4),
        ),
        ("Burning Sun Cavalry", "Colossadactyl", None, (3, 3)),
    ] {
        let mut game = scenario(player);
        let source = candidate(&mut game, player, name, Zone::Battlefield);
        let first = put(&mut game, player, first, Zone::Battlefield);
        if let Some(second) = second {
            put(&mut game, player, second, Zone::Battlefield);
        }
        let mut absent = game.clone();
        absent.move_object(first, Zone::Graveyard);
        attack(&mut absent, &[source]);
        assert!(absent.state.stack.is_empty(), "{name}");
        attack(&mut game, &[source]);
        game.move_object(first, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(stats(&game, source), expected, "{name}");
    }

    let mut block = scenario(player);
    let attacker = put(&mut block, player, "Hill Giant", Zone::Battlefield);
    let cavalry = candidate(
        &mut block,
        player.other(),
        "Burning Sun Cavalry",
        Zone::Battlefield,
    );
    put(
        &mut block,
        player.other(),
        "Colossadactyl",
        Zone::Battlefield,
    );
    attack(&mut block, &[attacker]);
    pass_pair(&mut block);
    act(
        &mut block,
        Action::Block {
            blocker: cavalry,
            attacker,
        },
    );
    act(&mut block, Action::FinishBlockers);
    settle(&mut block);
    assert_eq!(stats(&block, cavalry), (3, 3));
}

#[test]
fn breeches_reserves_each_mode_once_across_four_pirate_attackers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = candidate(
        &mut game,
        player,
        "Breeches, Eager Pillager",
        Zone::Battlefield,
    );
    let pirate = put(&mut game, player, "Goblin Boarders", Zone::Battlefield);
    let second = put(&mut game, player, "Goblin Boarders", Zone::Battlefield);
    let changeling = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    let blocker = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let exiled = put(&mut game, player, "Forest", Zone::Library);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    attack(&mut game, &[source, pirate, second, changeling]);
    for mode in 0..3 {
        choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
        for used in 0..mode {
            reject(&mut game, Action::ChooseMode { mode: used });
        }
        act(&mut game, Action::ChooseMode { mode });
        if mode == 1 {
            act(&mut game, target(blocker));
        }
    }
    assert_eq!(game.state.stack.len(), 3);
    settle(&mut game);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    assert!(
        game.object_view(blocker)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
    assert_eq!(game.object_view(exiled).zone, Zone::Exile);
    reject(&mut game, Action::PlayLand { card: exiled });
    game.begin_step(Step::Main2);
    act(&mut game, Action::PlayLand { card: exiled });
    assert_eq!(game.object_view(exiled).zone, Zone::Battlefield);
}

#[test]
fn bringer_returns_only_prior_creature_cards_after_a_cast_entry() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bringer = candidate(&mut game, player, "Bringer of the Last Gift", Zone::Hand);
    let doomed = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let finality = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let previous = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    game.add_counter(finality, CounterKind::Finality, 1);
    let mut uncast = game.clone();
    uncast.move_object(bringer, Zone::Battlefield);
    settle(&mut uncast);
    assert_eq!(uncast.object_view(doomed).zone, Zone::Battlefield);
    assert_eq!(uncast.object_view(previous).zone, Zone::Graveyard);
    cast(&mut game, bringer, None);
    settle(&mut game);
    assert!(
        game.object_view(bringer)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(game.object_view(doomed).zone, Zone::Graveyard);
    assert_eq!(game.object_view(finality).zone, Zone::Exile);
    assert_eq!(game.object_view(previous).zone, Zone::Battlefield);
    assert_eq!(game.object_view(foreign).controller, player.other());
    assert_eq!(game.object_view(foreign).zone, Zone::Battlefield);
}

#[test]
fn captain_storm_targets_a_controlled_pirate_for_each_artifact_in_the_batch() {
    let player = Player::P0;
    let mut game = scenario(player);
    let captain = candidate(
        &mut game,
        player,
        "Captain Storm, Cosmium Raider",
        Zone::Battlefield,
    );
    let wrong = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(
        &mut game,
        player.other(),
        "Goblin Boarders",
        Zone::Battlefield,
    );
    game.create_token_batch_capture(player, TREASURE, 2, false, None, true);
    game.prepare_triggers();
    choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
    reject(&mut game, target(wrong));
    reject(&mut game, target(other));
    for _ in 0..2 {
        act(&mut game, target(captain));
    }
    settle(&mut game);
    assert_eq!(game.counter_count(captain, CounterKind::PlusOne), 2);
}

#[test]
fn contested_game_ball_transfers_on_combat_then_pays_out_at_five_points() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ball = candidate(
        &mut game,
        player.other(),
        "Contested Game Ball",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.tap_objects(&[ball]);
    attack(&mut game, &[first, second]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.object_view(ball).controller, player);
    assert!(!game.object_view(ball).tapped);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    game.add_counter(ball, CounterKind::Point, 3);
    act(&mut game, activate(ball));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(ball, CounterKind::Point), 4);
    assert_eq!(game.hand_count(player), 1);
    assert!(tokens(&game, player, TREASURE).is_empty());
    game.untap_objects(&[ball], untap_rules::UntapCause::Effect);
    act(&mut game, activate(ball));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(ball).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
}

#[test]
fn deeproot_pilgrimage_observes_one_batch_of_nontoken_merfolk_taps() {
    let player = Player::P1;
    let mut game = scenario(player);
    candidate(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    let first = put(&mut game, player, "Brineborn Cutthroat", Zone::Battlefield);
    let second = put(&mut game, player, "Brineborn Cutthroat", Zone::Battlefield);
    let other = put(
        &mut game,
        player.other(),
        "Brineborn Cutthroat",
        Zone::Battlefield,
    );
    let token = game.create_token_batch_capture(player, MERFOLK, 1, false, None, true)[0];
    game.tap_objects(&[token.card, other]);
    assert!(game.state.pending_triggers.is_empty());
    attack(&mut game, &[first, second]);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let created = tokens(&game, player, MERFOLK)
        .into_iter()
        .find(|o| o.reference != token)
        .unwrap();
    assert_eq!(stats(&game, created.id), (1, 1));
    assert_eq!(created.colors, 2);
    assert!(created.has_subtype("Merfolk"));
    assert!(created.keywords.contains(Keywords::HEXPROOF));
    assert!(!created.tapped);
    assert_eq!(tokens(&game, player, MERFOLK).len(), 2);
}
