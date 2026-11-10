//! Edge of Eternities card scenarios for collector numbers 001 through 014.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!(
        "scenario did not reach its next choice at {:?}",
        game.state.pending
    );
}

fn choose_target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn trigger_target(game: &mut Game, card: CardId) {
    let action = choose_target(card);
    advance(game, |game| game.legal_actions().contains(&action));
    act(game, action);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(game: &mut Game, card: CardId, ability: u16, target: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(recipient) = target {
        act(game, choose_target(recipient));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn warp(game: &mut Game, card: CardId) {
    act(game, Action::AnnounceWarp { card });
    pay(game);
}

fn end_step(game: &mut Game) {
    game.state.step = Step::Main2;
    game.state.priority = game.state.active;
    pass_pair(game);
    assert_eq!(game.state.step, Step::End);
}

fn battlefield_tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectRef> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.token.is_some()
        })
        .map(|object| object.reference)
        .collect()
}

#[test]
fn anticausal_vestige_draws_before_offering_private_tapped_entry_on_warp_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Plains", Zone::Battlefield);
    put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let entrant = put(&mut game, player, "Bear Cub", Zone::Library);
    let expensive = put(&mut game, player, "Hill Giant", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    let vestige = put(&mut game, player, "Anticausal Vestige", Zone::Hand);

    warp(&mut game, vestige);
    settle(&mut game);
    assert_eq!(stats(&game, vestige), (7, 5));
    end_step(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(vestige).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).warped_cards[0].object,
        game.object_ref(vestige)
    );
    pass_pair(&mut game);

    assert_eq!(game.object_view(entrant).zone, Zone::Hand);
    let object = game.object_ref(entrant);
    assert_eq!(
        game.observe(player).effect_choice.unwrap().candidates,
        [object]
    );
    assert!(game.observe(player.other()).effect_choice.is_none());
    for card in [expensive, instant] {
        let object = game.object_ref(card);
        reject(&mut game, Action::PutHandPermanent { object });
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineHandEntry);
    assert_eq!(decline.object_view(entrant).zone, Zone::Hand);
    act(&mut game, Action::PutHandPermanent { object });
    assert_eq!(game.object_view(entrant).zone, Zone::Battlefield);
    assert!(game.object_view(entrant).tapped);
}

#[test]
fn tezzeret_uses_all_three_loyalty_abilities_and_artifact_entries_fund_them() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tezzeret = put(&mut game, player, "Tezzeret, Cruel Captain", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let drone = put(
        &mut game,
        player.other(),
        "Dockworker Drone",
        Zone::Battlefield,
    );
    let trail = put(&mut game, player, "Candy Trail", Zone::Library);
    let expensive = put(&mut game, player, "Leyline Axe", Zone::Library);
    cast(&mut game, tezzeret, None);
    settle(&mut game);
    assert_eq!(game.object_view(tezzeret).counters.loyalty, 4);
    cast(&mut game, bow, None);
    settle(&mut game);
    assert_eq!(game.object_view(tezzeret).counters.loyalty, 5);

    for (recipient, expected) in [(drone, 1), (bow, 0)] {
        let mut branch = game.clone();
        branch.state.objects[recipient as usize].tapped = true;
        activate(&mut branch, tezzeret, 0, Some(recipient));
        settle(&mut branch);
        assert!(!branch.object_view(recipient).tapped);
        assert_eq!(branch.object_view(recipient).counters.plus_one, expected);
    }

    let mut search = game.clone();
    activate(&mut search, tezzeret, 1, None);
    pass_pair(&mut search);
    assert_eq!(search.object_view(tezzeret).counters.loyalty, 2);
    reject(&mut search, Action::ChooseSearch { card: expensive });
    act(&mut search, Action::ChooseSearch { card: trail });
    act(&mut search, Action::FinishSearch);
    assert_eq!(search.object_view(trail).zone, Zone::Hand);

    let exact = game.object_ref(tezzeret);
    game.activation_cost_counters(exact, CounterKind::Loyalty, 3, player);
    activate(&mut game, tezzeret, 2, None);
    settle(&mut game);
    assert_eq!(game.object_view(tezzeret).counters.loyalty, 1);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(bow))
    });
    reject(&mut game, choose_target(drone));
    trigger_target(&mut game, bow);
    settle(&mut game);
    let robot = game.object_view(bow);
    assert!(robot.has_subtype("Equipment"));
    assert!(robot.has_subtype("Robot"));
    assert_eq!(robot.kinds & (1 | 16), 1 | 16);
    assert_eq!(robot.counters.plus_one, 3);
    assert_eq!(stats(&game, bow), (3, 3));
}

#[test]
fn all_fates_stalker_can_decline_and_returns_its_exact_exile_when_warped_away() {
    let player = Player::P1;
    let mut game = scenario(player);
    let stalker = put(&mut game, player, "All-Fates Stalker", Zone::Hand);
    let assassin = put(
        &mut game,
        player.other(),
        "All-Fates Stalker",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    warp(&mut game, stalker);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(bear))
    });
    reject(&mut game, choose_target(assassin));

    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(bear).zone, Zone::Battlefield);

    let original = game.object_ref(bear);
    trigger_target(&mut game, bear);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Exile);
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(stalker).zone, Zone::Exile);
    assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(bear), original);
}

#[test]
fn astelli_reclaimer_targets_using_the_exact_normal_or_warp_payment() {
    let player = Player::P0;
    let mut baseline = scenario(player);
    let reclaimer = put(&mut baseline, player, "Astelli Reclaimer", Zone::Hand);
    let bow = put(&mut baseline, player, "Short Bow", Zone::Graveyard);
    let axe = put(&mut baseline, player, "Leyline Axe", Zone::Graveyard);
    let bear = put(&mut baseline, player, "Bear Cub", Zone::Graveyard);
    let land = put(&mut baseline, player, "Forest", Zone::Graveyard);
    let shock = put(&mut baseline, player, "Shock", Zone::Graveyard);
    let foreign = put(&mut baseline, player.other(), "Short Bow", Zone::Graveyard);

    for warped in [false, true] {
        let mut game = baseline.clone();
        if warped {
            warp(&mut game, reclaimer);
        } else {
            cast(&mut game, reclaimer, None);
        }
        pass_pair(&mut game);
        let eligible = if warped { bow } else { axe };
        advance(&mut game, |game| {
            game.legal_actions().contains(&choose_target(eligible))
        });
        for card in [bear, land, shock, foreign] {
            reject(&mut game, choose_target(card));
        }
        if warped {
            reject(&mut game, choose_target(axe));
        }
        trigger_target(&mut game, eligible);
        settle(&mut game);
        assert_eq!(game.object_view(eligible).zone, Zone::Battlefield);
        assert!(
            game.object_view(reclaimer)
                .keywords
                .contains(Keywords::FLYING)
        );
    }
}

#[test]
fn auxiliary_boosters_attaches_to_one_created_robot_and_can_reequip() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Auxiliary Boosters", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player, "Exalted Sunborn", Zone::Battlefield);

    cast(&mut game, equipment, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let robots = battlefield_tokens(&game, player, Card(5309));
    assert_eq!(robots.len(), 2);
    let object = game.object_ref(bear);
    reject(&mut game, Action::ChooseCreatedToken { object });
    act(&mut game, Action::ChooseCreatedToken { object: robots[0] });
    settle(&mut game);
    assert_eq!(game.object_view(equipment).attached_to, Some(robots[0]));
    assert_eq!(stats(&game, robots[0].card), (3, 4));
    assert_eq!(stats(&game, robots[1].card), (2, 2));
    assert!(
        game.object_view(robots[0].card)
            .keywords
            .contains(Keywords::FLYING)
    );

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    activate(&mut game, equipment, 0, Some(bear));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (3, 4));
    assert!(game.object_view(bear).keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, robots[0].card), (2, 2));
    assert!(
        !game
            .object_view(robots[0].card)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn banishing_light_returns_only_the_exiled_opposing_nonland() {
    let player = Player::P1;
    let mut game = scenario(player);
    let light = put(&mut game, player, "Banishing Light", Zone::Hand);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);

    cast(&mut game, light, None);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(enemy))
    });
    reject(&mut game, choose_target(ally));
    reject(&mut game, choose_target(land));
    trigger_target(&mut game, enemy);
    let mut early_departure = game.clone();
    cast(&mut early_departure, disenchant, Some(Target::Card(light)));
    settle(&mut early_departure);
    assert_eq!(early_departure.object_view(enemy).zone, Zone::Battlefield);

    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
    cast(&mut game, disenchant, Some(Target::Card(light)));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy).controller, player.other());
}

#[test]
fn beyond_the_quiet_exiles_both_stationed_and_noncreature_spacecraft() {
    let player = Player::P0;
    let mut game = scenario(player);
    let quiet = put(&mut game, player, "Beyond the Quiet", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let stationed = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let spacecraft = put(
        &mut game,
        player.other(),
        "Warmaker Gunship",
        Zone::Battlefield,
    );
    let equipment = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let exact = game.object_ref(stationed);
    game.activation_cost_counters(exact, CounterKind::Charge, 6, player);
    assert_ne!(game.object_view(stationed).kinds & 1, 0);
    assert_eq!(game.object_view(spacecraft).kinds & 1, 0);
    let generation = game.object_ref(stationed).generation;

    cast(&mut game, quiet, None);
    settle(&mut game);
    for card in [creature, stationed, spacecraft] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert_eq!(game.object_ref(stationed).generation, generation + 1);
    for card in [equipment, land] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
}

#[test]
fn brightspear_zealot_counts_casts_before_resolution_and_attacks_untapped() {
    let player = Player::P0;
    let mut game = scenario(player);
    let zealot = put(&mut game, player, "Brightspear Zealot", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let negate = put(&mut game, player.other(), "Negate", Zone::Hand);
    cast(&mut game, bow, None);
    settle(&mut game);
    assert_eq!(stats(&game, zealot), (2, 4));
    cast(&mut game, shock, Some(Target::Player(player.other())));
    assert_eq!(stats(&game, zealot), (4, 4));
    if game.actor() == Some(player) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, negate, Some(Target::Spell(shock)));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 20);
    assert_eq!(stats(&game, zealot), (4, 4));

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: zealot });
    act(&mut game, Action::FinishAttackers);
    assert!(game.object_view(zealot).attacking);
    assert!(!game.object_view(zealot).tapped);
    advance(&mut game, |game| game.state.turn == 4);
    assert_eq!(stats(&game, zealot), (2, 4));
}

#[test]
fn cosmogrand_zenith_selects_either_mode_above_the_second_spell_only() {
    let player = Player::P0;
    let mut game = scenario(player);
    let zenith = put(&mut game, player, "Cosmogrand Zenith", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Hand);
    let third = put(&mut game, player, "Short Bow", Zone::Hand);
    cast(&mut game, zenith, None);
    settle(&mut game);
    cast(&mut game, second, None);

    advance(&mut game, |game| {
        game.legal_actions()
            .contains(&Action::ChooseMode { mode: 0 })
    });
    for mode in [0, 1] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseMode { mode });
        settle(&mut branch);
        assert_eq!(
            battlefield_tokens(&branch, player, Card(5241)).len(),
            if mode == 0 { 2 } else { 0 }
        );
        for (card, expected) in [(zenith, mode), (ally, mode), (enemy, 0), (second, 0)] {
            assert_eq!(
                branch.object_view(card).counters.plus_one,
                u64::from(expected)
            );
        }
        cast(&mut branch, third, None);
        settle(&mut branch);
        assert_eq!(branch.object_view(ally).counters.plus_one, u64::from(mode));
    }
}

#[test]
fn dawnstrike_vanguard_rechecks_two_tapped_allies_and_only_counters_other_creatures() {
    let player = Player::P1;
    let mut game = scenario(player);
    let vanguard = put(&mut game, player, "Dawnstrike Vanguard", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let shore_up = put(&mut game, player, "Shore Up", Zone::Hand);
    game.state.objects[vanguard as usize].tapped = true;
    game.state.objects[opponent as usize].tapped = true;

    let mut insufficient = game.clone();
    end_step(&mut insufficient);
    assert!(insufficient.state.stack.is_empty());
    game.state.objects[ally as usize].tapped = true;
    end_step(&mut game);
    assert_eq!(game.state.stack.len(), 1);

    let mut response = game.clone();
    response.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut response, shore_up, Some(Target::Card(ally)));
    settle(&mut response);
    assert!(!response.object_view(ally).tapped);
    assert_eq!(response.object_view(second).counters.plus_one, 0);

    settle(&mut game);
    assert_eq!(game.object_view(vanguard).counters.plus_one, 0);
    assert_eq!(game.object_view(opponent).counters.plus_one, 0);
    assert_eq!(game.object_view(ally).counters.plus_one, 1);
    assert_eq!(game.object_view(second).counters.plus_one, 1);
    assert!(
        game.object_view(vanguard)
            .keywords
            .contains(Keywords::LIFELINK)
    );
}

#[test]
fn dockworker_drone_copies_all_counter_kinds_from_the_dying_incarnation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let drone = put(&mut game, player, "Dockworker Drone", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, drone, None);
    settle(&mut game);
    assert_eq!(stats(&game, drone), (2, 2));
    assert_eq!(game.object_view(drone).counters.plus_one, 1);

    let exact = game.object_ref(drone);
    game.activation_cost_counters(exact, CounterKind::Charge, 3, player);
    game.activation_cost_counters(exact, CounterKind::Stun, 2, player);
    cast(&mut game, disenchant, Some(Target::Card(drone)));
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(ally))
    });
    reject(&mut game, choose_target(enemy));
    trigger_target(&mut game, ally);
    assert_eq!(game.object_view(drone).zone, Zone::Graveyard);
    settle(&mut game);
    let counters = game.object_view(ally).counters;
    assert_eq!(counters.plus_one, 1);
    assert_eq!(counters.stun, 2);
    assert_eq!(counters.get(CounterKind::Charge), 3);
}

#[test]
fn dual_sun_adepts_grow_existing_allies_and_deal_damage_twice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let adepts = put(&mut game, player, "Dual-Sun Adepts", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let later = put(&mut game, player, "Bear Cub", Zone::Hand);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 5];

    activate(&mut game, adepts, 0, None);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        2
    );
    assert_eq!(stats(&game, adepts), (3, 3));
    assert_eq!(stats(&game, ally), (3, 3));
    assert_eq!(stats(&game, opponent), (2, 2));
    // The unrestricted {5} payment may have consumed green mana.
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 0];
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(stats(&game, later), (2, 2));

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: adepts });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| {
        game.state.players[player.other().index()].life == 17
    });
    advance(&mut game, |game| {
        game.state.players[player.other().index()].life == 14
    });
    advance(&mut game, |game| game.state.turn == 4);
    assert_eq!(stats(&game, adepts), (2, 2));
}

#[test]
fn dual_sun_technique_checks_its_legal_target_for_a_plus_one_counter_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let technique = put(&mut game, player, "Dual-Sun Technique", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let drawn = *game.state.players[player.index()].library.last().unwrap();
    act(&mut game, Action::AnnounceCast { card: technique });
    reject(&mut game, choose_target(enemy));
    act(&mut game, choose_target(ally));
    pay(&mut game);
    let exact = game.object_ref(ally);
    game.activation_cost_counters(exact, CounterKind::Charge, 1, player);

    let mut no_counter = game.clone();
    settle(&mut no_counter);
    assert_eq!(no_counter.object_view(drawn).zone, Zone::Library);
    assert!(
        no_counter
            .object_view(ally)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );

    game.activation_cost_counters(exact, CounterKind::PlusOne, 1, player);
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert!(
        game.object_view(ally)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
}

#[test]
fn emergency_eject_gives_the_targets_controller_a_lander_even_if_destruction_fails() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Emergency Eject", Zone::Hand);
    let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let exact = game.object_ref(target);
    game.change_controller(exact, player.other());
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, choose_target(land));
    act(&mut game, choose_target(target));
    pay(&mut game);

    for indestructible in [false, true] {
        let mut branch = game.clone();
        if indestructible {
            branch.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        settle(&mut branch);
        assert_eq!(
            branch.object_view(target).zone,
            if indestructible {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert!(battlefield_tokens(&branch, player, Card(5238)).is_empty());
        let lander = battlefield_tokens(&branch, player.other(), Card(5238));
        assert_eq!(lander.len(), 1);
        if branch.actor() == Some(player) {
            act(&mut branch, Action::Pass);
        }
        activate(&mut branch, lander[0].card, 0, None);
        pass_pair(&mut branch);
        let basic = *branch.state.players[player.other().index()]
            .library
            .last()
            .unwrap();
        act(&mut branch, Action::ChooseSearch { card: basic });
        act(&mut branch, Action::FinishSearch);
        settle(&mut branch);
        assert_eq!(branch.object_view(basic).zone, Zone::Battlefield);
        assert!(branch.object_view(basic).tapped);
    }
}
