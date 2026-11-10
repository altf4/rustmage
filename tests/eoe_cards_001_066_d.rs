//! Edge of Eternities card scenarios for collector numbers 043 through 054.
#[path = "eoe_cards_001_066/scenario.rs"]
mod common;
mod support;

use common::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn wedgelight_rammer_creates_a_robot_that_can_station_it_despite_summoning_sickness() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Wedgelight Rammer", Zone::Hand);
    cast(&mut game, ship, None);
    settle(&mut game);
    let robots = battlefield_tokens(&game, player, Card(5309));
    assert_eq!(robots.len(), 1);
    let robot = game.object_view(robots[0].card);
    assert_eq!(robot.colors, 0);
    assert_eq!(robot.kinds & 17, 17);
    assert_eq!(stats(&game, robot.id), (2, 2));
    assert!(robot.has_subtype("Robot") && robot.summoning_sick);

    game.add_counter(ship, CounterKind::Charge, 7);
    assert!(!game.is_creature(ship));
    station(&mut game, ship, robot.id);
    settle(&mut game);
    assert!(game.object_view(robot.id).tapped);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 9);
    assert_eq!(stats(&game, ship), (3, 4));
    let keywords = game.object_view(ship).keywords;
    assert!(keywords.contains(Keywords::FLYING.union(Keywords::FIRST_STRIKE)));
    game.pay_counters(ship, &Counters::of(CounterKind::Charge, 1));
    assert!(!game.is_creature(ship));
}

#[test]
fn weftblade_enhancer_can_decline_and_uses_the_surviving_target_of_two() {
    let player = Player::P1;
    let mut game = scenario(player);
    let enhancer = put(&mut game, player, "Weftblade Enhancer", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    warp(&mut game, enhancer);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(enemy))
    });

    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    settle(&mut zero);
    assert_eq!(zero.object_view(enemy).counters.plus_one, 0);

    act(&mut game, choose_target(ally));
    trigger_target(&mut game, enemy);
    let mut both = game.clone();
    settle(&mut both);
    assert_eq!(both.object_view(ally).counters.plus_one, 1);
    assert_eq!(both.object_view(enemy).counters.plus_one, 1);

    cast(&mut game, bounce, Some(Target::Card(ally)));
    settle(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Hand);
    assert_eq!(game.object_view(enemy).counters.plus_one, 1);
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(enhancer).zone, Zone::Exile);
}

#[test]
fn zealous_display_untaps_its_boosted_creatures_only_on_an_opponents_turn() {
    let player = Player::P0;
    for active in [player, player.other()] {
        let mut game = scenario(active);
        let display = put(&mut game, player, "Zealous Display", Zone::Hand);
        let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        for card in [ally, enemy] {
            game.state.objects[card as usize].tapped = true;
        }
        opponent_priority(&mut game, player);
        cast(&mut game, display, None);
        settle(&mut game);
        assert_eq!(stats(&game, ally), (4, 2));
        assert_eq!(stats(&game, enemy), (2, 2));
        assert_eq!(game.object_view(ally).tapped, active == player);
        assert!(game.object_view(enemy).tapped);
    }
}

#[test]
fn annul_accepts_artifact_and_enchantment_spells_and_excludes_other_creature_spells() {
    let player = Player::P0;
    for name in ["Short Bow", "Banishing Light", "Bear Cub"] {
        let mut game = scenario(player.other());
        let spell = put(&mut game, player.other(), name, Zone::Hand);
        let annul = put(&mut game, player, "Annul", Zone::Hand);
        cast(&mut game, spell, None);
        opponent_priority(&mut game, player);
        if name == "Bear Cub" {
            reject(&mut game, Action::AnnounceCast { card: annul });
            settle(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        } else {
            cast(&mut game, annul, Some(Target::Spell(spell)));
            settle(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn atomic_microsizer_sets_base_stats_preserving_boosts_and_prevents_a_legal_block() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Atomic Microsizer", Zone::Battlefield);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(attacker, CounterKind::PlusOne, 1);
    activate(&mut game, equipment, 0, Some(attacker));
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (5, 4));

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    for card in [attacker, other] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    trigger_target(&mut game, attacker);
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (3, 2));
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    reject(&mut game, Action::Block { blocker, attacker });
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: other,
        },
    );
}

#[test]
fn cerebral_download_samples_artifacts_on_resolution_and_surveys_privately_before_drawing() {
    let player = Player::P0;
    let mut game = scenario(player);
    let download = put(&mut game, player, "Cerebral Download", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    put(&mut game, player, "Dockworker Drone", Zone::Battlefield);
    put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    let kept = put(&mut game, player, "Plains", Zone::Library);
    let milled = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, download, None);
    let mut fewer = game.clone();
    cast(&mut fewer, removal, Some(Target::Card(bow)));
    pass_pair(&mut fewer);
    pass_pair(&mut fewer);
    assert_eq!(
        fewer
            .observe(player)
            .library_choice
            .unwrap()
            .remaining
            .len(),
        1
    );

    pass_pair(&mut game);
    let choice = game.observe(player).library_choice.unwrap();
    assert!(choice.surveil);
    assert_eq!(choice.remaining.len(), 2);
    assert!(game.observe(player.other()).library_choice.is_none());
    let old_hand = game.hand_count(player);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: milled,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut game);
    assert_eq!(game.hand_count(player), old_hand + 3);
    assert_eq!(game.object_view(kept).zone, Zone::Hand);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
}

#[test]
fn cloudsculpt_technician_gains_only_one_power_while_its_controller_has_an_artifact() {
    let player = Player::P1;
    let mut game = scenario(player);
    let technician = put(
        &mut game,
        player,
        "Cloudsculpt Technician",
        Zone::Battlefield,
    );
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, technician), (1, 4));
    assert!(game
        .object_view(technician)
        .keywords
        .contains(Keywords::FLYING));
    cast(&mut game, bow, None);
    settle(&mut game);
    assert_eq!(stats(&game, technician), (2, 4));
    let exact = game.object_ref(foreign);
    game.change_controller(exact, player);
    assert_eq!(stats(&game, technician), (2, 4));
    game.move_object(bow, Zone::Graveyard);
    game.change_controller(exact, player.other());
    assert_eq!(stats(&game, technician), (1, 4));
}

#[test]
fn codecracker_hound_privately_requires_one_of_two_and_mills_the_other() {
    let player = Player::P1;
    let mut game = scenario(player);
    let hound = put(&mut game, player, "Codecracker Hound", Zone::Hand);
    let first = put(&mut game, player, "Plains", Zone::Library);
    let second = put(&mut game, player, "Island", Zone::Library);
    warp(&mut game, hound);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let choice = game.observe(player).library_selection.unwrap();
    assert_eq!(choice.candidates.len(), 2);
    assert_eq!(choice.minimum, 1);
    assert!(!choice.reveal && !choice.public);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::FinishLibrarySelection);
    for (keep, mill) in [(first, second), (second, first)] {
        let mut branch = game.clone();
        act(&mut branch, Action::SelectLibrary { card: keep });
        act(&mut branch, Action::FinishLibrarySelection);
        assert_eq!(branch.object_view(keep).zone, Zone::Hand);
        assert_eq!(branch.object_view(mill).zone, Zone::Graveyard);
        end_step(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(hound).zone, Zone::Exile);
    }
}

#[test]
fn consult_the_star_charts_looks_at_its_controllers_lands_and_selects_for_the_kicked_cost() {
    let player = Player::P0;
    for kicked in [false, true] {
        let mut game = scenario(player);
        let charts = put(&mut game, player, "Consult the Star Charts", Zone::Hand);
        for owner in [player, player, player, player.other()] {
            let land = put(&mut game, owner, "Forest", Zone::Battlefield);
            game.state.objects[land as usize].tapped = true;
        }
        let first = put(&mut game, player, "Plains", Zone::Library);
        let second = put(&mut game, player, "Island", Zone::Library);
        let third = put(&mut game, player, "Swamp", Zone::Library);
        game.state.players[player.index()].mana = if kicked {
            [0, 2, 0, 0, 0, 2]
        } else {
            [0, 1, 0, 0, 0, 1]
        };
        act(&mut game, Action::AnnounceCast { card: charts });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        pay(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        pass_pair(&mut game);
        assert_eq!(
            game.observe(player)
                .library_selection
                .unwrap()
                .remaining
                .len(),
            3
        );
        assert!(game.observe(player.other()).library_selection.is_none());
        act(&mut game, Action::SelectLibrary { card: first });
        if kicked {
            reject(&mut game, Action::FinishLibrarySelection);
            act(&mut game, Action::SelectLibrary { card: second });
        }
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(first).zone, Zone::Hand);
        assert_eq!(
            game.object_view(second).zone,
            if kicked { Zone::Hand } else { Zone::Library }
        );
        assert!(
            game.state.players[player.index()].library[..if kicked { 1 } else { 2 }]
                .contains(&third)
        );
    }
}

#[test]
fn cryogen_relic_draws_on_entry_and_sacrifice_then_stuns_an_optional_tapped_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let relic = put(&mut game, player, "Cryogen Relic", Zone::Hand);
    let tapped = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let untapped = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Island", Zone::Library);
    let first = put(&mut game, player, "Plains", Zone::Library);
    game.state.objects[tapped as usize].tapped = true;
    cast(&mut game, relic, None);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: relic,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, choose_target(untapped));

    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    pay(&mut no_target);
    settle(&mut no_target);
    assert_eq!(no_target.object_view(relic).zone, Zone::Graveyard);
    assert_eq!(no_target.object_view(second).zone, Zone::Hand);
    assert_eq!(no_target.object_view(tapped).counters.stun, 0);

    act(&mut game, choose_target(tapped));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.object_view(tapped).counters.stun, 0);
    settle(&mut game);
    assert_eq!(game.object_view(tapped).counters.stun, 1);
}

#[test]
fn cryoshatter_destroys_its_captured_tapped_or_damaged_subject_even_after_leaving() {
    let player = Player::P0;
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Cryoshatter", Zone::Hand);
    let donor = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(donor)));
    settle(&mut game);
    assert_eq!(stats(&game, donor), (5, 10));

    let mut damaged = game.clone();
    cast(&mut damaged, shock, Some(Target::Card(donor)));
    pass_pair(&mut damaged);
    assert_eq!(damaged.object_view(donor).damage, 2);
    assert_eq!(
        damaged.state.stack.last().unwrap().kind,
        StackKind::TriggeredAbility { ability: 1 }
    );
    cast(&mut damaged, disenchant, Some(Target::Card(aura)));
    settle(&mut damaged);
    assert_eq!(damaged.object_view(donor).zone, Zone::Graveyard);

    station(&mut game, ship, donor);
    assert_eq!(
        game.state.stack.last().unwrap().kind,
        StackKind::TriggeredAbility { ability: 0 }
    );
    settle(&mut game);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 5);
}

#[test]
fn desculpting_blast_returns_to_the_owner_and_creates_a_drone_only_for_an_attacker() {
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(enemy);
    let blast = put(&mut game, player, "Desculpting Blast", Zone::Hand);
    let stolen = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let flying = put(&mut game, enemy, "Healer's Hawk", Zone::Battlefield);
    let ground = put(&mut game, enemy, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, enemy, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, enemy, "Forest", Zone::Battlefield);
    let exact = game.object_ref(stolen);
    game.change_controller(exact, enemy);
    game.state.objects[stolen as usize].summoning_sick = false;
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    for card in [stolen, flying, ground] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    opponent_priority(&mut game, player);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: blast });
    reject(&mut game, choose_target(land));

    let mut nonattacker = game.clone();
    act(&mut nonattacker, choose_target(artifact));
    pay(&mut nonattacker);
    settle(&mut nonattacker);
    assert_eq!(nonattacker.object_view(artifact).zone, Zone::Hand);
    assert!(battlefield_tokens(&nonattacker, player, Card(5537)).is_empty());

    act(&mut game, choose_target(stolen));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, player);
    let drones = battlefield_tokens(&game, player, Card(5537));
    assert_eq!(drones.len(), 1);
    let drone = drones[0].card;
    let view = game.object_view(drone);
    assert_eq!(view.kinds & 17, 17);
    assert!(view.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, drone), (1, 1));
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: drone,
            attacker: ground,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: drone,
            attacker: flying,
        },
    );
}
