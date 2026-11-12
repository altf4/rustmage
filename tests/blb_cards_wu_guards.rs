mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn step(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::Pass | Action::PutTrigger { .. })
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..24 {
        if game.legal_actions().contains(&action) {
            return;
        }
        step(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn attack(game: &mut Game, card: CardId) {
    let action = Action::Attack { card };
    until(game, action.clone());
    act(game, action);
    act(game, Action::FinishAttackers);
}

fn token_count(game: &Game, player: Player, subtype: &str) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && o.has_subtype(subtype)
        })
        .count()
}

#[test]
fn beza_evaluates_four_strict_comparisons_as_it_resolves() {
    let player = Player::P0;
    for behind in [false, true] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        if behind {
            put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
            put(&mut game, opponent, "Forest", Zone::Battlefield);
            put(&mut game, opponent, "Forest", Zone::Hand);
            game.lose_life(player, 1);
        }
        let beza = put(&mut game, player, "Beza, the Bounding Spring", Zone::Hand);
        cast(&mut game, beza, None);
        settle(&mut game);
        assert_eq!(stats(&game, beza), (4, 5));
        assert_eq!(token_count(&game, player, "Treasure"), usize::from(behind));
        assert_eq!(
            token_count(&game, player, "Fish"),
            if behind { 2 } else { 0 }
        );
        assert_eq!(
            game.state.players[player.index()].life,
            if behind { 23 } else { 20 }
        );
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(behind)
        );
    }
}

#[test]
fn mabels_mettle_uses_two_distinct_roles_and_resolves_a_surviving_second_role() {
    let player = Player::P1;
    let mut game = scenario(player);
    let large = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let small = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Mabel's Mettle", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(large));
    reject(&mut game, target(large));
    let mut one = game.clone();
    act(&mut one, Action::FinishTargets);
    pay(&mut one);
    settle(&mut one);
    assert_eq!(stats(&one, large), (4, 4));
    assert_eq!(stats(&one, small), (2, 2));

    act(&mut game, target(small));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    game.move_object(large, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, small), (3, 3));
}

#[test]
fn pileated_provisioner_rechecks_the_targets_flying_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let flier = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let ground = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let provisioner = put(&mut game, player, "Pileated Provisioner", Zone::Hand);
    cast(&mut game, provisioner, None);
    until(&mut game, target(ground));
    reject(&mut game, target(flier));
    reject(&mut game, target(opposing));
    act(&mut game, target(ground));
    let mut becomes_flying = game.clone();
    let exact = becomes_flying.object_ref(ground);
    becomes_flying.grant_keyword_effects(&[ground], exact, Keywords::FLYING);
    settle(&mut becomes_flying);
    assert_eq!(
        becomes_flying.counter_count(ground, CounterKind::PlusOne),
        0
    );
    settle(&mut game);
    assert_eq!(game.counter_count(ground, CounterKind::PlusOne), 1);
    assert!(
        game.object_view(provisioner)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn sonar_strike_checks_combat_state_and_samples_the_bat_when_resolving() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bat = put(&mut game, player, "Wax-Wane Witness", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Sonar Strike", Zone::Hand);
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::Cast { card, target: Some(Target::Card(id)), .. } if *card == spell && *id == victim
    )));
    game.state.objects[victim as usize].tapped = true;
    cast(&mut game, spell, Some(Target::Card(victim)));
    let mut no_bat = game.clone();
    no_bat.move_object(bat, Zone::Graveyard);
    settle(&mut no_bat);
    assert_eq!(no_bat.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(no_bat.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(stats(&game, bat), (3, 4));
}

#[test]
fn star_charter_and_wax_wane_see_actual_life_events_even_at_net_zero() {
    let player = Player::P0;
    let mut game = scenario(player);
    let witness = put(&mut game, player, "Wax-Wane Witness", Zone::Battlefield);
    let charter = put(&mut game, player, "Star Charter", Zone::Battlefield);
    let big = put(&mut game, player, "Serra Angel", Zone::Library);
    let small = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let mut no_changes = game.clone();
    no_changes.begin_step(Step::End);
    no_changes.prepare_triggers();
    settle(&mut no_changes);
    assert!(no_changes.observe(player).library_selection.is_none());

    game.gain_life(player, 1);
    game.lose_life(player, 1);
    game.prepare_triggers();
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(&mut game, action);
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(stats(&game, witness), (4, 4));
    assert!(
        game.object_view(witness)
            .keywords
            .contains(Keywords::VIGILANCE.union(Keywords::FLYING))
    );
    assert!(
        game.object_view(charter)
            .keywords
            .contains(Keywords::FLYING)
    );
    game.begin_step(Step::End);
    game.prepare_triggers();
    until(&mut game, Action::SelectLibrary { card: small });
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: big });
    act(&mut game, Action::SelectLibrary { card: small });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(small).zone, Zone::Hand);
    assert_eq!(game.object_view(big).zone, Zone::Library);
}

#[test]
fn warrenguard_uses_token_control_at_declaration_and_warden_uses_earlier_entry() {
    let player = Player::P0;
    for qualified in [false, true] {
        let mut guard = scenario(player);
        let warrior = put(
            &mut guard,
            player,
            "Seasoned Warrenguard",
            Zone::Battlefield,
        );
        let mut token = None;
        if qualified {
            token = Some(guard.state.objects.len() as CardId);
            guard.create_tokens(player, Card(5246), 1, false);
        }
        attack(&mut guard, warrior);
        if let Some(token) = token {
            guard.move_object(token, Zone::Exile);
        }
        settle(&mut guard);
        assert_eq!(stats(&guard, warrior), (if qualified { 3 } else { 1 }, 2));

        let mut warden = scenario(player);
        let frog = put(&mut warden, player, "Waterspout Warden", Zone::Battlefield);
        if qualified {
            let entrant = put(&mut warden, player, "Bear Cub", Zone::Hand);
            cast(&mut warden, entrant, None);
            settle(&mut warden);
            warden.move_object(entrant, Zone::Graveyard);
        }
        attack(&mut warden, frog);
        settle(&mut warden);
        assert_eq!(
            warden.object_view(frog).keywords.contains(Keywords::FLYING),
            qualified
        );
    }
}

#[test]
fn calamitous_tide_loots_with_zero_or_partially_invalid_targets() {
    let player = Player::P0;
    for two in [false, true] {
        let mut game = scenario(player);
        let a = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let b = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let top = put(&mut game, player, "Opt", Zone::Library);
        let spell = put(&mut game, player, "Calamitous Tide", Zone::Hand);
        let targets = [Target::Card(a), Target::Card(b)];
        announce(&mut game, spell, if two { &targets } else { &[] });
        if two {
            game.move_object(a, Zone::Exile);
        }
        let discard = Action::Discard { card: top };
        until(&mut game, discard.clone());
        act(&mut game, discard);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        assert_eq!(game.object_view(top).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(b).zone,
            if two { Zone::Hand } else { Zone::Battlefield }
        );
    }
}

#[test]
fn dazzling_denial_demands_four_for_a_live_bird_and_two_when_it_leaves() {
    let player = Player::P1;
    let caster = player.other();
    let mut game = scenario(caster);
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let denial = put(&mut game, player, "Dazzling Denial", Zone::Hand);
    let giant = put(&mut game, caster, "Hill Giant", Zone::Hand);
    cast(&mut game, giant, None);
    act(&mut game, Action::Pass);
    cast(&mut game, denial, Some(Target::Spell(giant)));
    let mut lost_bird = game.clone();
    lost_bird.move_object(bird, Zone::Graveyard);
    for (branch, generic) in [(&mut game, 4), (&mut lost_bird, 2)] {
        branch.state.players[caster.index()].mana = [0, 0, 0, 0, 0, 4];
        let payment = Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, generic],
        };
        until(branch, payment.clone());
        if generic == 4 {
            reject(
                branch,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 2],
                },
            );
        }
        act(branch, payment);
        settle(branch);
        assert_eq!(branch.object_view(giant).zone, Zone::Battlefield);
        assert_eq!(branch.state.players[caster.index()].mana[5], 4 - generic);
    }
}

#[test]
fn mindwhisker_surveil_crosses_threshold_and_thought_shucker_limits_its_paid_use() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mind = put(&mut game, player, "Mindwhisker", Zone::Battlefield);
    let thought = put(&mut game, player, "Thought Shucker", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let top = put(&mut game, player, "Forest", Zone::Library);
    for _ in 0..6 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    let announce = Action::AnnounceActivate {
        card: thought,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, announce.clone());
    assert_eq!(stats(&game, enemy), (2, 2));
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    let surveil = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Graveyard,
    };
    until(&mut game, surveil.clone());
    act(&mut game, surveil);
    assert_eq!(stats(&game, enemy), (1, 2));
    assert_eq!(stats(&game, mind), (3, 2));
    choose(&mut game, |a| {
        matches!(a,
            Action::Activate { card, ability: 0, .. } | Action::AnnounceActivate { card, ability: 0, .. } if *card == thought
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    game.move_object(top, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, thought), (2, 4));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, enemy), (2, 2));
    put(&mut game, player, "Forest", Zone::Graveyard);
    reject(&mut game, announce);
}

#[test]
fn shoreline_looter_always_draws_and_only_discards_below_threshold() {
    let player = Player::P0;
    for threshold in [false, true] {
        let mut game = scenario(player);
        let looter = put(&mut game, player, "Shoreline Looter", Zone::Battlefield);
        let top = put(&mut game, player, "Forest", Zone::Library);
        if threshold {
            for _ in 0..7 {
                put(&mut game, player, "Forest", Zone::Graveyard);
            }
        }
        assert!(
            game.object_view(looter)
                .keywords
                .contains(Keywords::CANT_BE_BLOCKED)
        );
        attack(&mut game, looter);
        until(&mut game, Action::FinishBlockers);
        act(&mut game, Action::FinishBlockers);
        if !threshold {
            let discard = Action::Discard { card: top };
            until(&mut game, discard.clone());
            act(&mut game, discard);
        } else {
            for _ in 0..8 {
                if game.state.step == Step::CombatDamage {
                    break;
                }
                act(&mut game, Action::Pass);
            }
            assert_eq!(game.state.step, Step::CombatDamage);
        }
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            game.object_view(top).zone,
            if threshold {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
}
