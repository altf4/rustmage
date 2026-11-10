//! Edge of Eternities card scenarios for collector numbers 015 through 028.
#[path = "eoe_cards_001_066/scenario.rs"]
mod common;
mod support;

use common::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn sunborn_doubles_knight_luminary_on_both_cast_routes_and_lifelinks_in_combat() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sunborn = put(&mut game, player, "Exalted Sunborn", Zone::Battlefield);
    let knight = put(&mut game, player, "Knight Luminary", Zone::Hand);
    let ground = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    warp(&mut game, knight);
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, player, Card(5241)).len(), 2);

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: sunborn });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: ground,
            attacker: sunborn,
        },
    );
    advance(&mut game, |game| {
        game.state.players[player.index()].life == 24
    });
    assert_eq!(game.state.players[player.other().index()].life, 16);

    advance(&mut game, |game| {
        game.object_view(knight).zone == Zone::Exile
    });
    let exact = game.object_ref(knight);
    assert_eq!(game.observe(player).warped_cards[0].object, exact);
    advance(&mut game, |game| {
        game.state.active == player && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 3];
    act(&mut game, Action::AnnounceExile { card: knight });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, player, Card(5241)).len(), 4);
    assert!(game.observe(player).warped_cards.is_empty());
    end_step(&mut game);
    assert_eq!(game.object_view(knight).zone, Zone::Battlefield);
}

#[test]
fn exalted_sunborn_warps_and_exiles_its_own_incarnation() {
    let player = Player::P1;
    let mut game = scenario(player);
    let sunborn = put(&mut game, player, "Exalted Sunborn", Zone::Hand);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    warp(&mut game, sunborn);
    settle(&mut game);
    assert_eq!(stats(&game, sunborn), (4, 5));
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(sunborn).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).warped_cards[0].object,
        game.object_ref(sunborn)
    );
}

#[test]
fn exosuit_savior_can_decline_or_return_a_stolen_permanent_to_its_owner() {
    let player = Player::P1;
    let mut game = scenario(player);
    let savior = put(&mut game, player, "Exosuit Savior", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let stolen = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let exact = game.object_ref(stolen);
    game.change_controller(exact, player);
    cast(&mut game, savior, None);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(stolen))
    });
    assert!(game.legal_actions().contains(&choose_target(land)));
    reject(&mut game, choose_target(savior));
    reject(&mut game, choose_target(foreign));
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(stolen).zone, Zone::Battlefield);
    trigger_target(&mut game, stolen);
    settle(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, player.other());
    assert!(game.object_view(savior).keywords.contains(Keywords::FLYING));
}

#[test]
fn flight_deck_coordinator_counts_station_taps_and_rechecks_on_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let coordinator = put(
        &mut game,
        player,
        "Flight-Deck Coordinator",
        Zone::Battlefield,
    );
    let donor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let shore_up = put(&mut game, player, "Shore Up", Zone::Hand);
    game.state.objects[coordinator as usize].tapped = true;
    game.state.objects[opposing as usize].tapped = true;

    let mut one = game.clone();
    end_step(&mut one);
    assert!(one.state.stack.is_empty());
    station(&mut game, ship, donor);
    settle(&mut game);
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let mut untapped = game.clone();
    untapped.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut untapped, shore_up, Some(Target::Card(donor)));
    settle(&mut untapped);
    assert_eq!(untapped.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn focus_fire_counts_the_live_union_of_creatures_and_spacecraft_during_combat() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fire = put(&mut game, player, "Focus Fire", Zone::Hand);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let blocker = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    let bystander = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let stationed = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    game.add_counter(stationed, CounterKind::Charge, 6);
    assert!(game.is_creature(stationed));
    assert!(!game.is_creature(ship));

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    game.state.players[player.index()].mana = [1, 1, 0, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: fire });
    assert!(game.legal_actions().contains(&choose_target(attacker)));
    reject(&mut game, choose_target(bystander));
    act(&mut game, choose_target(blocker));
    pay(&mut game);

    let mut fewer = game.clone();
    cast(&mut fewer, bounce, Some(Target::Card(stationed)));
    settle(&mut fewer);
    assert_eq!(fewer.object_view(blocker).damage, 4);
    settle(&mut game);
    assert_eq!(game.object_view(blocker).damage, 5);
}

#[test]
fn haliya_counts_artifact_creatures_once_and_can_meet_the_draw_condition_in_response() {
    let player = Player::P1;
    let mut game = scenario(player);
    let haliya = put(&mut game, player, "Haliya, Guided by Light", Zone::Hand);
    let drone = put(&mut game, player, "Dockworker Drone", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let opposing = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let strike = put(&mut game, player, "Radiant Strike", Zone::Hand);
    let top = *game.state.players[player.index()].library.last().unwrap();
    warp(&mut game, haliya);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);

    let mut three = game.clone();
    for card in [drone, bow] {
        cast(&mut three, card, None);
        settle(&mut three);
    }
    assert_eq!(three.state.players[player.index()].life, 23);

    end_step(&mut game);
    advance(&mut game, |game| game.state.pending == Pending::Priority);
    assert_eq!(game.state.stack.len(), 2);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 3];
    cast(&mut game, strike, Some(Target::Card(opposing)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.object_view(haliya).zone, Zone::Exile);
}

#[test]
fn hardlight_containment_attaches_to_your_artifact_and_grants_ward_until_it_leaves() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let containment = put(&mut game, player, "Hardlight Containment", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let other_bow = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let prisoner = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let removal = put(&mut game, opponent, "Disenchant", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: containment });
    reject(&mut game, choose_target(other_bow));
    act(&mut game, choose_target(bow));
    pay(&mut game);
    pass_pair(&mut game);
    trigger_target(&mut game, prisoner);
    settle(&mut game);
    assert_eq!(game.object_view(prisoner).zone, Zone::Exile);
    assert_eq!(
        game.object_view(containment).attached_to,
        Some(game.object_ref(bow))
    );

    opponent_priority(&mut game, opponent);
    cast(&mut game, removal, Some(Target::Card(bow)));
    advance(&mut game, |game| {
        game.legal_actions()
            .contains(&Action::DeclineResolutionMana)
    });
    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(bow).zone, Zone::Battlefield);
    assert_eq!(unpaid.object_view(prisoner).zone, Zone::Exile);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Graveyard);
    assert_eq!(game.object_view(containment).zone, Zone::Graveyard);
    assert_eq!(game.object_view(prisoner).zone, Zone::Battlefield);
}

#[test]
fn honor_can_target_an_opposing_creature_but_skips_its_draw_if_the_target_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let honor = put(&mut game, player, "Honor", Zone::Hand);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let top = *game.state.players[player.index()].library.last().unwrap();
    cast(&mut game, honor, Some(Target::Card(enemy)));

    let mut stale = game.clone();
    cast(&mut stale, bounce, Some(Target::Card(enemy)));
    settle(&mut stale);
    assert_eq!(stale.object_view(enemy).zone, Zone::Hand);
    assert_eq!(stale.object_view(top).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.object_view(enemy).counters.plus_one, 1);
}

#[test]
fn honored_knight_captain_creates_a_soldier_and_sacrifices_before_searching() {
    let player = Player::P0;
    let mut game = scenario(player);
    let captain = put(&mut game, player, "Honored Knight-Captain", Zone::Hand);
    let equipment = put(&mut game, player, "Short Bow", Zone::Library);
    let other = put(&mut game, player, "Candy Trail", Zone::Library);
    cast(&mut game, captain, None);
    settle(&mut game);
    let soldiers = battlefield_tokens(&game, player, Card(5241));
    assert_eq!(soldiers.len(), 1);
    let soldier = game.object_view(soldiers[0].card);
    assert_eq!(soldier.colors, 1);
    assert_eq!(stats(&game, soldier.id), (1, 1));
    assert!(soldier.has_subtype("Human") && soldier.has_subtype("Soldier"));

    game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 4];
    activate(&mut game, captain, 0, None);
    assert_eq!(game.object_view(captain).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: other });
    let mut failed_search = game.clone();
    act(&mut failed_search, Action::FinishSearch);
    assert_eq!(failed_search.object_view(equipment).zone, Zone::Library);
    act(&mut game, Action::ChooseSearch { card: equipment });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert!(game.object_view(equipment).attached_to.is_none());
}

#[test]
fn lightstall_inquisitor_privately_exiles_an_opponents_choice_with_exact_play_terms() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let inquisitor = put(&mut game, player, "Lightstall Inquisitor", Zone::Hand);
    let land = put(&mut game, opponent, "Island", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    cast(&mut game, inquisitor, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    let options = game.observe(opponent).effect_choice.unwrap().candidates;
    assert_eq!(options, [game.object_ref(land), game.object_ref(creature)]);
    assert!(
        game.object_view(inquisitor)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert!(!game.legal_actions().contains(&Action::DeclineEffectPayment));

    for chosen in [land, creature] {
        let mut branch = game.clone();
        let object = branch.object_ref(chosen);
        act(&mut branch, Action::SelectResolutionExile { object });
        branch.move_object(inquisitor, Zone::Graveyard);
        advance(&mut branch, |game| {
            game.state.active == opponent && game.state.step == Step::Main1
        });
        let exact = branch.object_ref(chosen);
        assert_eq!(branch.observe(player).exile_permissions[0].object, exact);
        if chosen == land {
            reject(&mut branch, Action::PlayLand { card: land });
            act(&mut branch, Action::PlayTaxedExileLand { object: exact });
            assert!(branch.object_view(land).tapped);
        } else {
            branch.state.players[opponent.index()].mana = [0, 0, 0, 0, 1, 1];
            let taxed = Action::AnnounceTaxedExile {
                object: exact,
                face: CastFace::Normal,
                method: CastMethod::Normal,
                flash_surcharge: false,
                surcharge: std::num::NonZeroU32::new(1).unwrap(),
            };
            reject(&mut branch, taxed.clone());
            branch.state.players[opponent.index()].mana[5] += 1;
            act(&mut branch, taxed);
            pay(&mut branch);
            settle(&mut branch);
        }
        assert_eq!(branch.object_view(chosen).zone, Zone::Battlefield);
        let hidden = if chosen == land { creature } else { land };
        assert_eq!(branch.object_view(hidden).zone, Zone::Hand);
        assert!(
            branch
                .observe(player)
                .objects
                .iter()
                .all(|object| object.id != hidden)
        );
    }
}

#[test]
fn lumen_class_frigate_crosses_its_two_station_thresholds_independently() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Lumen-Class Frigate", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let dinosaur = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, ship, None);
    settle(&mut game);
    assert!(!game.is_creature(ship));
    assert_eq!(stats(&game, bear), (2, 2));

    station(&mut game, ship, bear);
    settle(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 2);
    assert_eq!(stats(&game, bear), (3, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
    assert!(!game.is_creature(ship));
    assert_eq!(stats(&game, dinosaur), (11, 11));

    station(&mut game, ship, dinosaur);
    settle(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 13);
    assert_eq!(stats(&game, ship), (3, 5));
    let keywords = game.object_view(ship).keywords;
    assert!(keywords.contains(Keywords::FLYING.union(Keywords::LIFELINK)));
    game.pay_counters(ship, &Counters::of(CounterKind::Charge, 2));
    assert!(!game.is_creature(ship));
    assert_eq!(stats(&game, bear), (3, 3));
    game.pay_counters(ship, &Counters::of(CounterKind::Charge, 10));
    assert_eq!(stats(&game, bear), (2, 2));
}

#[test]
fn luxknight_breacher_counts_each_other_controlled_creature_or_artifact_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let breacher = put(&mut game, player, "Luxknight Breacher", Zone::Hand);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    put(&mut game, player, "Dockworker Drone", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Dockworker Drone",
        Zone::Battlefield,
    );
    put(&mut game, player, "Forest", Zone::Battlefield);
    cast(&mut game, breacher, None);
    settle(&mut game);
    assert_eq!(game.object_view(breacher).counters.plus_one, 3);
    assert_eq!(stats(&game, breacher), (5, 5));
}

#[test]
fn pinnacle_starcage_converts_only_its_remaining_linked_cards_and_sacrifices() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cage = put(&mut game, player, "Pinnacle Starcage", Zone::Hand);
    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let bear = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let drone = put(
        &mut game,
        player.other(),
        "Dockworker Drone",
        Zone::Battlefield,
    );
    let sunborn = put(&mut game, player, "Exalted Sunborn", Zone::Battlefield);
    cast(&mut game, cage, None);
    settle(&mut game);
    for card in [bow, bear, drone] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }

    let mut released = game.clone();
    cast(&mut released, removal, Some(Target::Card(cage)));
    settle(&mut released);
    for card in [bow, bear, drone] {
        assert_eq!(released.object_view(card).zone, Zone::Battlefield);
    }

    game.move_object(drone, Zone::Hand);
    game.move_object(drone, Zone::Exile);
    game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 6];
    activate(&mut game, cage, 0, None);
    assert_eq!(game.object_view(cage).zone, Zone::Battlefield);
    settle(&mut game);
    for card in [cage, bow, bear] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(drone).zone, Zone::Exile);
    assert_eq!(game.object_view(sunborn).zone, Zone::Battlefield);
    assert_eq!(battlefield_tokens(&game, player, Card(5309)).len(), 4);
}

#[test]
fn pulsar_squadron_ace_offers_a_private_spacecraft_and_owns_its_decline_branch() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ace = put(&mut game, player, "Pulsar Squadron Ace", Zone::Hand);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Library);
    let miss = put(&mut game, player, "Short Bow", Zone::Library);
    cast(&mut game, ace, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let choice = game.observe(player).library_selection.unwrap();
    assert_eq!(choice.remaining.len(), 5);
    assert_eq!(choice.candidates, [game.object_ref(ship)]);
    assert!(choice.reveal && !choice.public);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: miss });

    let mut declined = game.clone();
    act(&mut declined, Action::FinishLibrarySelection);
    settle(&mut declined);
    assert_eq!(declined.object_view(ace).counters.plus_one, 1);
    assert_eq!(declined.object_view(ship).zone, Zone::Library);

    let select = Action::SelectLibrary { card: ship };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == select)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, select);
    act(&mut indexed, Action::FinishLibrarySelection);
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut indexed);
    settle(&mut game);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(ace).counters.plus_one, 0);
    assert_eq!(game.object_view(ship).zone, Zone::Hand);
    assert!(game.state.players[player.index()].library[..4].contains(&miss));
}
