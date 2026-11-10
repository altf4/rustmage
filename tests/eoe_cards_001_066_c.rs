//! Edge of Eternities card scenarios for collector numbers 029 through 042.
#[path = "eoe_cards_001_066/scenario.rs"]
mod common;
mod support;

use common::*;
use rustmage as _;
use rustmage_core::{
    cards::authored::{CardExtras, CardRules},
    cards::*,
    engine::*,
};
use support::*;

fn counter_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseCreatureCounterCost {
        object: game.object_ref(card),
        counter: CounterKind::PlusOne,
    }
}

fn spell_mode(game: &mut Game, spell: CardId, mode: u16, target: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    act(game, choose_target(target));
    pay(game);
}

#[test]
fn radiant_strike_gains_life_on_indestructible_artifacts_but_not_on_an_illegal_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let strike = put(&mut game, player, "Radiant Strike", Zone::Hand);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let untap = put(&mut game, player.other(), "Shore Up", Zone::Hand);
    game.state.objects[artifact as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    act(&mut game, Action::AnnounceCast { card: strike });
    reject(&mut game, choose_target(creature));

    let mut artifact_branch = game.clone();
    act(&mut artifact_branch, choose_target(artifact));
    pay(&mut artifact_branch);
    settle(&mut artifact_branch);
    assert_eq!(
        artifact_branch.object_view(artifact).zone,
        Zone::Battlefield
    );
    assert_eq!(artifact_branch.state.players[player.index()].life, 23);

    game.state.objects[creature as usize].tapped = true;
    act(&mut game, choose_target(creature));
    pay(&mut game);
    opponent_priority(&mut game, player.other());
    cast(&mut game, untap, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 20);
}

#[test]
fn rayblade_trooper_triggers_on_its_own_marked_nontoken_death_and_rejects_other_events() {
    let player = Player::P0;
    let mut game = scenario(player);
    let trooper = put(&mut game, player, "Rayblade Trooper", Zone::Hand);
    let luminary = put(&mut game, player, "Knight Luminary", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, luminary, None);
    settle(&mut game);
    let token = battlefield_tokens(&game, player, Card(5241))[0].card;
    game.add_counter(token, CounterKind::PlusOne, 1);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    game.add_counter(ally, CounterKind::Charge, 1);

    warp(&mut game, trooper);
    pass_pair(&mut game);
    trigger_target(&mut game, trooper);
    settle(&mut game);
    assert_eq!(stats(&game, trooper), (3, 3));
    for (dead, soldiers_after) in [(trooper, 2), (token, 0), (enemy, 1), (ally, 1)] {
        let mut branch = game.clone();
        cast(&mut branch, murder, Some(Target::Card(dead)));
        settle(&mut branch);
        assert_eq!(
            battlefield_tokens(&branch, player, Card(5241)).len(),
            soldiers_after
        );
    }
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(trooper).zone, Zone::Exile);
    assert_eq!(battlefield_tokens(&game, player, Card(5241)).len(), 1);
}

#[test]
fn reroute_systems_can_protect_an_artifact_or_damage_a_still_tapped_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Reroute Systems", Zone::Hand);
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    let untap = put(&mut game, player, "Shore Up", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[bear as usize].tapped = true;

    let mut protection = game.clone();
    cast(&mut protection, destroy, Some(Target::Card(artifact)));
    spell_mode(&mut protection, spell, 0, artifact);
    settle(&mut protection);
    assert_eq!(protection.object_view(artifact).zone, Zone::Battlefield);
    assert!(
        protection
            .object_view(artifact)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );

    spell_mode(&mut game, spell, 1, bear);
    let mut untapped = game.clone();
    cast(&mut untapped, untap, Some(Target::Card(bear)));
    settle(&mut untapped);
    assert_eq!(untapped.object_view(bear).damage, 0);
    assert_eq!(untapped.object_view(bear).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
}

#[test]
fn rescue_skiff_returns_an_aura_without_targeting_its_recipient_and_stations() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Rescue Skiff", Zone::Hand);
    let aura = put(&mut game, player, "Ethereal Armor", Zone::Graveyard);
    let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let equipment = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let donor = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
    let recipient = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let exact_recipient = game.object_ref(recipient);
    game.state.objects[recipient as usize].gained_keywords = Keywords::HEXPROOF;

    cast(&mut game, ship, None);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(aura))
    });
    reject(&mut game, choose_target(equipment));
    let mut creature_branch = game.clone();
    trigger_target(&mut creature_branch, creature);
    settle(&mut creature_branch);
    assert_eq!(
        creature_branch.object_view(creature).zone,
        Zone::Battlefield
    );

    trigger_target(&mut game, aura);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseAuraAttachment {
            object: exact_recipient,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(exact_recipient));
    assert!(!game.is_creature(ship));
    station(&mut game, ship, donor);
    settle(&mut game);
    assert_eq!(stats(&game, ship), (5, 6));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn scout_for_survivors_enforces_a_joint_mana_value_and_counters_only_returned_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scout = put(&mut game, player, "Scout for Survivors", Zone::Hand);
    let first = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let second = put(&mut game, player, "Starport Security", Zone::Graveyard);
    let third = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let too_much = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: scout });

    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    pay(&mut zero);
    settle(&mut zero);
    assert_eq!(zero.object_view(first).zone, Zone::Graveyard);

    act(&mut game, choose_target(first));
    act(&mut game, choose_target(second));
    reject(&mut game, choose_target(too_much));
    act(&mut game, choose_target(third));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    game.move_object(third, Zone::Exile);
    settle(&mut game);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(card).counters.plus_one, 1);
    }
    assert_eq!(game.object_view(third).zone, Zone::Exile);
    assert_eq!(game.object_view(third).counters.plus_one, 0);
}

#[test]
fn seam_rip_binds_only_a_small_opposing_nonland_and_returns_it_on_departure() {
    let player = Player::P1;
    let mut game = scenario(player);
    let seam = put(&mut game, player, "Seam Rip", Zone::Hand);
    let own = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let small = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, seam, None);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(small))
    });
    for card in [own, large, land] {
        reject(&mut game, choose_target(card));
    }
    trigger_target(&mut game, small);
    settle(&mut game);
    assert_eq!(game.object_view(small).zone, Zone::Exile);
    cast(&mut game, removal, Some(Target::Card(seam)));
    settle(&mut game);
    assert_eq!(game.object_view(small).zone, Zone::Battlefield);
}

#[test]
fn the_seriema_searches_for_a_legendary_creature_and_protects_only_tapped_legendary_allies() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "The Seriema", Zone::Hand);
    let legendary = put(&mut game, player, "Haliya, Guided by Light", Zone::Library);
    let walker = put(&mut game, player, "Tezzeret, Cruel Captain", Zone::Library);
    let common = put(&mut game, player, "Bear Cub", Zone::Library);
    let donor = put(
        &mut game,
        player,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    game.state.objects[ally as usize].tapped = true;
    game.state.objects[enemy as usize].tapped = true;

    cast(&mut game, ship, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for card in [walker, common] {
        reject(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::ChooseSearch { card: legendary });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(legendary).zone, Zone::Hand);

    let indestructible = |game: &Game, card| {
        game.object_view(card)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    };
    assert!(!indestructible(&game, donor));
    station(&mut game, ship, donor);
    settle(&mut game);
    assert_eq!(stats(&game, ship), (5, 5));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
    game.state.objects[ship as usize].tapped = true;
    assert!(indestructible(&game, donor));
    for card in [ally, enemy, ship] {
        assert!(!indestructible(&game, card));
    }
    let mut untapped = game.clone();
    untapped.state.objects[donor as usize].tapped = false;
    assert!(!indestructible(&untapped, donor));
    game.pay_counters(ship, &Counters::of(CounterKind::Charge, 6));
    assert!(!indestructible(&game, donor));
    assert!(!game.is_creature(ship));
}

#[test]
fn squires_lightblade_flashes_in_and_its_first_strike_outlasts_reequipping() {
    let player = Player::P0;
    for active in [player, player.other()] {
        let mut game = scenario(active);
        let blade = put(&mut game, player, "Squire's Lightblade", Zone::Hand);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        opponent_priority(&mut game, player);
        cast(&mut game, blade, None);
        pass_pair(&mut game);
        trigger_target(&mut game, first);
        settle(&mut game);
        assert_eq!(stats(&game, first), (3, 2));
        assert!(
            game.object_view(first)
                .keywords
                .contains(Keywords::FIRST_STRIKE)
        );
        if active == player {
            activate(&mut game, blade, 0, Some(second));
            settle(&mut game);
            assert_eq!(stats(&game, first), (2, 2));
            assert_eq!(stats(&game, second), (3, 2));
            assert!(
                game.object_view(first)
                    .keywords
                    .contains(Keywords::FIRST_STRIKE)
            );
            assert!(
                !game
                    .object_view(second)
                    .keywords
                    .contains(Keywords::FIRST_STRIKE)
            );
        }
    }
}

#[test]
fn starfield_shepherd_warps_and_reveals_either_a_basic_plains_or_a_small_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let shepherd = put(&mut game, player, "Starfield Shepherd", Zone::Hand);
    let plains = put(&mut game, player, "Plains", Zone::Library);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Library);
    let bear = put(&mut game, player, "Bear Cub", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    warp(&mut game, shepherd);
    pass_pair(&mut game);
    pass_pair(&mut game);
    for card in [bear, forest] {
        reject(&mut game, Action::ChooseSearch { card });
    }
    for card in [plains, hawk] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseSearch { card });
        act(&mut branch, Action::FinishSearch);
        assert_eq!(branch.object_view(card).zone, Zone::Hand);
        assert!(
            branch
                .object_view(shepherd)
                .keywords
                .contains(Keywords::FLYING)
        );
        end_step(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(shepherd).zone, Zone::Exile);
    }
}

#[test]
fn starfighter_pilot_surveils_privately_before_the_station_ability_resolves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let pilot = put(&mut game, player, "Starfighter Pilot", Zone::Battlefield);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let top = put(&mut game, player, "Island", Zone::Library);
    station(&mut game, ship, pilot);
    pass_pair(&mut game);
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(player.other()).library_choice.is_none());
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 0);
    for destination in [LibraryDestination::Top, LibraryDestination::Graveyard] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: top,
                destination,
            },
        );
        settle(&mut branch);
        let zone = if destination == LibraryDestination::Top {
            Zone::Library
        } else {
            Zone::Graveyard
        };
        assert_eq!(branch.object_view(top).zone, zone);
        assert_eq!(branch.counter_count(ship, CounterKind::Charge), 2);
    }
}

#[test]
fn starport_security_locks_the_discount_when_announcing_and_checks_the_exact_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let guard = put(&mut game, player, "Starport Security", Zone::Battlefield);
    let discount = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let activate = Action::AnnounceActivate {
        card: guard,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    game.add_counter(discount, CounterKind::Charge, 1);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    reject(&mut game, activate.clone());
    game.add_counter(discount, CounterKind::PlusOne, 1);
    game.state.objects[guard as usize].summoning_sick = true;
    reject(&mut game, activate.clone());
    game.state.objects[guard as usize].summoning_sick = false;
    act(&mut game, activate);
    reject(&mut game, choose_target(guard));
    act(&mut game, choose_target(enemy));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    game.pay_counters(discount, &Counters::of(CounterKind::PlusOne, 1));
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.object_view(guard).tapped);
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
}

#[test]
fn sunstar_chaplain_gives_a_counter_then_pays_that_counter_as_a_separate_cost() {
    let player = Player::P1;
    let mut game = scenario(player);
    let chaplain = put(&mut game, player, "Sunstar Chaplain", Zone::Battlefield);
    let donor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    for card in [chaplain, donor] {
        game.state.objects[card as usize].tapped = true;
    }
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    end_step(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(donor))
    });
    reject(&mut game, choose_target(enemy));
    trigger_target(&mut game, donor);
    settle(&mut game);
    assert_eq!(game.object_view(donor).counters.plus_one, 1);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, chaplain, 0, Some(artifact));
    let foreign = counter_cost(&game, enemy);
    reject(&mut game, foreign);
    let own = counter_cost(&game, donor);
    act(&mut game, own);
    pay(&mut game);
    assert_eq!(game.object_view(donor).counters.plus_one, 0);
    assert!(!game.object_view(artifact).tapped);
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
}

const COUNTER_MANA_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 2,
        ]))],
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_put_counters: &[(0, CounterKind::PlusOne, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn sunstar_counter_funding_prunes_land_taps_but_keeps_mana_that_adds_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let chaplain = put(&mut game, player, "Sunstar Chaplain", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    let activation = Action::AnnounceActivate {
        card: chaplain,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut lands = game.clone();
    for _ in 0..24 {
        put(&mut lands, player, "Plains", Zone::Battlefield);
    }
    assert!(!lands.legal_actions().contains(&activation));
    reject(&mut lands, activation.clone());
    let own = put(&mut lands, player, "Bear Cub", Zone::Battlefield);
    lands.add_counter(own, CounterKind::PlusOne, 1);
    activate(&mut lands, chaplain, 0, Some(enemy));
    let foreign = counter_cost(&lands, enemy);
    assert!(!lands.legal_actions().contains(&foreign));
    reject(&mut lands, foreign);
    let own_payment = counter_cost(&lands, own);
    act(&mut lands, own_payment);

    let _fixture = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_MANA_CREATURE)]);
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, activation);
    act(&mut game, choose_target(enemy));
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == donor),
    );
    assert_eq!(game.counter_count(donor, CounterKind::PlusOne), 1);
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    let payment = counter_cost(&game, donor);
    act(&mut game, payment);
    settle(&mut game);
    assert_eq!(game.counter_count(donor, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    assert!(game.object_view(enemy).tapped);
}

#[test]
fn sunstar_can_pay_an_existing_counter_or_one_returned_during_mana_payment() {
    let player = Player::P1;
    let mut game = scenario(player);
    let chaplain = put(&mut game, player, "Sunstar Chaplain", Zone::Battlefield);
    let existing = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let ring = put(&mut game, player, "Lotus Ring", Zone::Battlefield);
    let mascot = put(&mut game, player, "District Mascot", Zone::Hand);
    let stalker = put(&mut game, player, "All-Fates Stalker", Zone::Hand);
    game.add_counter(existing, CounterKind::PlusOne, 1);
    cast(&mut game, mascot, None);
    settle(&mut game);
    assert_eq!(game.counter_count(mascot, CounterKind::PlusOne), 1);
    cast(&mut game, stalker, None);
    pass_pair(&mut game);
    trigger_target(&mut game, mascot);
    settle(&mut game);
    let exiled = game.object_ref(mascot);
    assert_eq!(game.object_view(mascot).zone, Zone::Exile);
    game.state.objects[stalker as usize].summoning_sick = false;
    activate(&mut game, ring, 0, Some(stalker));
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];

    activate(&mut game, chaplain, 0, Some(enemy));
    let grantor = game.object_ref(ring);
    let sacrifice = Action::TapGrantedMana {
        card: stalker,
        grantor,
        production: [0, 0, 0, 0, 3, 0],
    };
    let current_payment = counter_cost(&game, existing);
    let mut existing_branch = game.clone();
    act(&mut existing_branch, current_payment.clone());
    act(&mut existing_branch, sacrifice.clone());
    pay(&mut existing_branch);
    settle(&mut existing_branch);
    assert_eq!(
        existing_branch.counter_count(existing, CounterKind::PlusOne),
        0
    );
    assert_eq!(
        existing_branch.counter_count(mascot, CounterKind::PlusOne),
        1
    );

    act(&mut game, Action::BeginOrderedCosts);
    act(&mut game, sacrifice);
    assert_eq!(game.object_view(stalker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(mascot).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(mascot), exiled);
    assert_eq!(game.counter_count(mascot, CounterKind::PlusOne), 1);
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    reject(
        &mut game,
        Action::ChooseCreatureCounterCost {
            object: exiled,
            counter: CounterKind::PlusOne,
        },
    );
    assert!(game.legal_actions().contains(&current_payment));
    let returned_payment = counter_cost(&game, mascot);
    act(&mut game, returned_payment);
    settle(&mut game);
    assert_eq!(game.counter_count(existing, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(mascot).zone, Zone::Graveyard);
    assert!(game.object_view(enemy).tapped);
}

#[test]
fn sunstar_expansionist_rechecks_land_counts_and_grows_from_lander_landfall() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scout = put(&mut game, player, "Sunstar Expansionist", Zone::Hand);
    let land = put(&mut game, player.other(), "Island", Zone::Battlefield);
    let prior = spawn(&mut game, player, Card(5238), Zone::Battlefield);
    let mut equal = game.clone();
    equal.move_object(land, Zone::Hand);
    cast(&mut equal, scout, None);
    settle(&mut equal);
    assert_eq!(battlefield_tokens(&equal, player, Card(5238)).len(), 1);

    cast(&mut game, scout, None);
    pass_pair(&mut game);
    let landfall = |game: &mut Game, token| {
        let basic = *game.state.players[player.index()].library.last().unwrap();
        activate(game, token, 0, None);
        pass_pair(game);
        act(game, Action::ChooseSearch { card: basic });
        act(game, Action::FinishSearch);
        settle(game);
        assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
        assert!(game.object_view(basic).tapped);
        assert_eq!(stats(game, scout), (3, 3));
    };

    let mut caught_up = game.clone();
    landfall(&mut caught_up, prior);
    assert!(battlefield_tokens(&caught_up, player, Card(5238)).is_empty());
    settle(&mut game);
    let created = battlefield_tokens(&game, player, Card(5238))
        .into_iter()
        .find(|token| token.card != prior)
        .unwrap();
    landfall(&mut game, created.card);
}

#[test]
fn sunstar_lightsmith_places_a_counter_on_the_second_cast_and_draws_even_if_it_leaves() {
    let player = Player::P1;
    let mut game = scenario(player);
    let smith = put(&mut game, player, "Sunstar Lightsmith", Zone::Battlefield);
    let first = put(&mut game, player, "Short Bow", Zone::Hand);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let top = *game.state.players[player.index()].library.last().unwrap();
    cast(&mut game, first, None);
    settle(&mut game);
    assert_eq!(stats(&game, smith), (3, 3));
    assert_eq!(game.object_view(top).zone, Zone::Library);
    cast(&mut game, second, Some(Target::Player(player.other())));

    let mut departed = game.clone();
    cast(&mut departed, bounce, Some(Target::Card(smith)));
    settle(&mut departed);
    assert_eq!(departed.object_view(smith).zone, Zone::Hand);
    assert_eq!(departed.object_view(top).zone, Zone::Hand);
    settle(&mut game);
    assert_eq!(stats(&game, smith), (4, 4));
    assert_eq!(game.object_view(top).zone, Zone::Hand);
}
