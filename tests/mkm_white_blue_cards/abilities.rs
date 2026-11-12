use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);
const DETECTIVE: Card = Card::from_catalogue_id(5329);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(game: &mut Game, source: CardId, slot: u16, recipient: Option<CardId>) {
    choose(game, |action| match action {
        Action::AnnounceActivate { card, ability, .. } => *card == source && *ability == slot,
        Action::Activate {
            card,
            ability,
            target,
            ..
        } => *card == source && *ability == slot && *target == recipient.map(Target::Card),
        _ => false,
    });
    if matches!(game.state.pending, Pending::Targets { .. }) {
        act(game, target(recipient.unwrap()));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == card)
        .collect()
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

#[test]
fn delney_doubles_inside_sources_entry_but_the_sorcery_activation_is_single() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Delney, Streetwise Lookout",
        Zone::Battlefield,
    );
    let large = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let insider = put(&mut game, player, "Inside Source", Zone::Hand);
    cast(&mut game, insider, None);
    pass_pair(&mut game);
    order_triggers(&mut game);
    settle(&mut game);
    let detectives = tokens(&game, player, DETECTIVE);
    assert_eq!(detectives.len(), 2);
    let recipient = detectives[0];
    assert!(game.object_view(recipient).has_subtype("Detective"));
    assert_eq!(game.object_view(recipient).colors, 3);
    let announce = Action::AnnounceActivate {
        card: insider,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, announce.clone());
    game.state.objects[insider as usize].summoning_sick = false;
    let mut combat = game.clone();
    combat.begin_step(Step::BeginCombat);
    reject(&mut combat, announce.clone());
    act(&mut game, announce);
    reject(&mut game, target(large));
    act(&mut game, target(recipient));
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert!(game.object_view(insider).tapped);
    assert_eq!(stats(&game, recipient), (4, 2));
    assert_eq!(stats(&game, detectives[1]), (2, 2));
    assert!(
        game.object_view(recipient)
            .keywords
            .contains(Keywords::VIGILANCE)
    );

    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[recipient as usize].summoning_sick = false;
    pass_to(&mut game, Pending::Attackers);
    for card in [small, recipient] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_to(&mut game, Pending::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: large,
            attacker: small,
        },
    );
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: large,
        attacker: recipient
    }));
}

#[test]
fn due_diligence_only_targets_a_different_creature_and_its_two_boosts_are_independent() {
    let player = Player::P0;
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Due Diligence", Zone::Hand);
    let enchanted = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, aura, Some(Target::Card(enchanted)));
    pass_pair(&mut game);
    reject(&mut game, target(enchanted));
    reject(&mut game, target(opposing));
    act(&mut game, target(second));
    settle(&mut game);
    for id in [enchanted, second] {
        assert_eq!(stats(&game, id), (4, 4));
        assert!(game.object_view(id).keywords.contains(Keywords::VIGILANCE));
    }
    game.move_object(aura, Zone::Graveyard);
    assert_eq!(stats(&game, enchanted), (2, 2));
    assert!(
        !game
            .object_view(enchanted)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert_eq!(stats(&game, second), (4, 4));
}

#[test]
fn haazda_vigilante_targets_live_small_creatures_both_on_entry_and_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vigilante = put(&mut game, player, "Haazda Vigilante", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, vigilante, None);
    pass_pair(&mut game);
    reject(&mut game, target(vigilante));
    act(&mut game, target(first));
    settle(&mut game);
    assert_eq!(stats(&game, first), (3, 3));

    game.state.objects[vigilante as usize].summoning_sick = false;
    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: vigilante });
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(first));
    act(&mut game, target(second));
    let mut too_large = game.clone();
    too_large.add_counter(second, CounterKind::PlusOne, 1);
    settle(&mut too_large);
    assert_eq!(stats(&too_large, second), (3, 3));
    settle(&mut game);
    assert_eq!(stats(&game, second), (3, 3));
}

#[test]
fn krovod_haunch_equips_and_its_sacrifice_offers_one_optional_two_dog_batch() {
    let player = Player::P0;
    let mut game = scenario(player);
    let haunch = put(&mut game, player, "Krovod Haunch", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, haunch, None);
    settle(&mut game);
    activate(&mut game, haunch, 1, Some(creature));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (4, 2));
    activate(&mut game, haunch, 0, None);
    assert_eq!(game.object_view(haunch).zone, Zone::Graveyard);
    assert_eq!(stats(&game, creature), (2, 2));
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    let dog = Card::from_catalogue_id(5256);
    assert!(tokens(&declined, player, dog).is_empty());
    assert_eq!(declined.state.players[player.index()].life, 23);

    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    let dogs = tokens(&game, player, dog);
    assert_eq!(dogs.len(), 2);
    for id in dogs {
        let token = game.object_view(id);
        assert!(token.has_subtype("Dog"));
        assert_eq!(token.colors, 1);
        assert_eq!(stats(&game, id), (1, 1));
    }
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn sanctuary_wall_taps_even_when_the_stun_payment_is_declined() {
    let player = Player::P1;
    let mut game = scenario(player);
    let wall = put(&mut game, player, "Sanctuary Wall", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    activate(&mut game, wall, 0, Some(enemy));
    pass_pair(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert!(game.object_view(wall).tapped);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.counter_count(enemy, CounterKind::Stun), 0);
    assert_eq!(declined.counter_count(wall, CounterKind::Stun), 0);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(wall, CounterKind::Stun), 1);
    assert!(game.object_view(wall).keywords.contains(Keywords::DEFENDER));
}

#[test]
fn wojek_investigator_checks_the_opponents_hand_only_when_its_upkeep_trigger_resolves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let investigator = put(&mut game, player, "Wojek Investigator", Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Hand);
    let draw = put(&mut game, opponent, "Quick Study", Zone::Hand);
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    let mut equal = game.clone();
    settle(&mut equal);
    assert!(tokens(&equal, player, CLUE).is_empty());
    act(&mut game, Action::Pass);
    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    let keywords = game.object_view(investigator).keywords;
    assert!(keywords.contains(Keywords::FLYING) && keywords.contains(Keywords::VIGILANCE));
}

#[test]
fn wrench_grants_the_bearer_a_separate_tap_ability_and_can_be_sacrificed_to_draw() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let wrench = put(&mut game, player, "Wrench", Zone::Hand);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, wrench, None);
    settle(&mut game);
    activate(&mut game, wrench, 2, Some(bearer));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (3, 3));
    assert!(
        game.object_view(bearer)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    game.record_control(game.object_ref(bearer), opponent, None);
    let action = Action::AnnounceAttachedAbility {
        grantor: game.object_ref(wrench),
        ability: 0,
    };
    reject(&mut game, action.clone());
    game.state.priority = opponent;
    game.state.objects[bearer as usize].summoning_sick = true;
    reject(&mut game, action.clone());
    game.state.objects[bearer as usize].summoning_sick = false;
    act(&mut game, action);
    act(&mut game, target(victim));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(bearer).tapped);
    assert!(!game.object_view(wrench).tapped);
    assert!(game.object_view(victim).tapped);

    game.state.priority = player;
    activate(&mut game, wrench, 1, None);
    settle(&mut game);
    assert_eq!(game.object_view(wrench).zone, Zone::Graveyard);
    assert_eq!(stats(&game, bearer), (2, 2));
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn curious_inquiry_grants_the_creatures_controller_a_clue_for_combat_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let inquiry = put(&mut game, player, "Curious Inquiry", Zone::Hand);
    let bearer = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, inquiry, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (3, 3));
    game.state.active = opponent;
    game.state.priority = opponent;
    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: bearer });
    act(&mut game, Action::FinishAttackers);
    pass_to(&mut game, Pending::Blockers);
    act(&mut game, Action::FinishBlockers);
    for _ in 0..8 {
        if game.state.players[player.index()].life < 20 {
            break;
        }
        act(&mut game, Action::Pass);
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(tokens(&game, opponent, CLUE).len(), 1);
    assert!(tokens(&game, player, CLUE).is_empty());
}

#[test]
fn dramatic_accusation_freezes_the_bearer_and_can_shuffle_it_using_attachment_lki() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let accusation = put(&mut game, player, "Dramatic Accusation", Zone::Hand);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[other as usize].tapped = true;
    cast(&mut game, accusation, Some(Target::Card(victim)));
    settle(&mut game);
    assert!(game.object_view(victim).tapped);
    game.state.active = opponent;
    game.untap_active_player();
    assert!(game.object_view(victim).tapped);
    assert!(!game.object_view(other).tapped);
    game.state.active = player;
    game.state.priority = player;
    activate(&mut game, accusation, 0, None);
    game.move_object(accusation, Zone::Graveyard);
    let mut stale = game.clone();
    stale.move_object(victim, Zone::Exile);
    stale.move_object(victim, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stale.object_view(victim).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Library);
    assert!(
        game.state.players[opponent.index()]
            .library
            .contains(&victim)
    );
}

#[test]
fn eliminate_the_impossible_only_shrinks_and_unsuspects_opposing_creatures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ours = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    for id in [ours, theirs] {
        game.set_suspected(game.object_ref(id), true);
    }
    let eliminate = put(&mut game, player, "Eliminate the Impossible", Zone::Hand);
    cast(&mut game, eliminate, None);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    assert_eq!(stats(&game, ours), (2, 2));
    assert_eq!(stats(&game, theirs), (0, 2));
    assert!(game.is_suspected(ours));
    assert!(!game.is_suspected(theirs));
    assert!(!game.object_view(theirs).keywords.contains(Keywords::MENACE));
}

#[test]
fn forensic_gadgeteer_discounts_artifact_activations_and_courier_observes_actual_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forensic Gadgeteer", Zone::Battlefield);
    let courier = put(&mut game, player, "Furtive Courier", Zone::Battlefield);
    let candle = put(&mut game, player, "Candlestick", Zone::Hand);
    cast(&mut game, candle, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(candle).zone, Zone::Stack);
    let clue = tokens(&game, player, CLUE)[0];
    settle(&mut game);
    put(&mut game, player, "Forensic Gadgeteer", Zone::Battlefield);
    assert!(
        !game
            .object_view(courier)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    let pay_one = Action::Activate {
        card: clue,
        ability: 0,
        target: None,
        payment: [0, 0, 0, 0, 0, 1],
        counter_payment: Counters::default(),
    };
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, pay_one.clone());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, pay_one);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert!(
        game.object_view(courier)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    pass_to(&mut game, Pending::Attackers);
    act(&mut game, Action::Attack { card: courier });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(player), 2);
    choose(&mut game, |action| matches!(action, Action::Discard { .. }));
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn hotshot_investigators_investigates_only_if_the_bounced_creature_was_controlled() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let hotshot = put(&mut game, player, "Hotshot Investigators", Zone::Hand);
    let stolen = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    cast(&mut game, hotshot, None);
    pass_pair(&mut game);
    reject(&mut game, target(hotshot));
    let mut skipped = game.clone();
    act(&mut skipped, Action::FinishTargets);
    settle(&mut skipped);
    assert!(tokens(&skipped, player, CLUE).is_empty());
    assert_eq!(skipped.object_view(stolen).zone, Zone::Battlefield);

    let mut enemy = game.clone();
    act(&mut enemy, target(opposing));
    settle(&mut enemy);
    assert!(tokens(&enemy, player, CLUE).is_empty());
    assert_eq!(enemy.object_view(opposing).zone, Zone::Hand);

    act(&mut game, target(stolen));
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, opponent);
}
