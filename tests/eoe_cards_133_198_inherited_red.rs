//! Definition-level scenarios for admitted EOE red cards in collectors 133–170.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(game: &mut Game, card: CardId, ability: u16) {
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
fn frontline_war_rager_rechecks_tapped_creatures_after_the_end_step_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rager = put(&mut game, player, "Frontline War-Rager", Zone::Battlefield);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let lands = [
        put(&mut game, player, "Forest", Zone::Battlefield),
        put(&mut game, player, "Forest", Zone::Battlefield),
    ];
    let burst = put(&mut game, player, "Biosynthic Burst", Zone::Hand);
    for id in [rager, other] {
        game.state.objects[id as usize].tapped = true;
    }
    until(&mut game, |game| game.state.step == Step::End);
    let mut keep = game.clone();
    settle(&mut keep);
    assert_eq!(keep.counter_count(rager, CounterKind::PlusOne), 1);
    for card in lands {
        act(&mut game, Action::TapMana { card });
    }
    cast(&mut game, burst, Some(Target::Card(other)));
    settle(&mut game);
    assert!(!game.object_view(other).tapped);
    assert_eq!(game.counter_count(rager, CounterKind::PlusOne), 0);
}

#[test]
fn full_bore_uses_the_exact_warp_receipt_and_team_leader_only_boosts_other_own_creatures() {
    let player = Player::P0;
    for warp in [false, true] {
        let mut game = scenario(player);
        let leader = put(&mut game, player, "Memorial Team Leader", Zone::Hand);
        let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let bore = put(&mut game, player, "Full Bore", Zone::Hand);
        if warp {
            act(&mut game, Action::AnnounceWarp { card: leader });
            pay(&mut game);
        } else {
            cast(&mut game, leader, None);
        }
        settle(&mut game);
        assert_eq!(stats(&game, ally), (3, 2));
        assert_eq!(stats(&game, leader), (4, 3));
        assert_eq!(stats(&game, enemy), (2, 2));
        cast(&mut game, bore, Some(Target::Card(leader)));
        settle(&mut game);
        assert_eq!(stats(&game, leader), (7, 5));
        let view = game.object_view(leader);
        assert_eq!(view.keywords.contains(Keywords::TRAMPLE), warp);
        assert_eq!(view.keywords.contains(Keywords::HASTE), warp);
        let turn = game.state.turn;
        until(&mut game, |game| game.state.turn == turn + 1);
        assert_eq!(stats(&game, ally), (2, 2));
        assert_eq!(
            game.object_view(leader).zone,
            if warp { Zone::Exile } else { Zone::Battlefield }
        );
        if !warp {
            assert_eq!(stats(&game, leader), (4, 3));
        }
    }
}

#[test]
fn kavaron_turbodrone_taps_for_growth_and_haste_and_molecular_modifier_fires_in_combat() {
    let player = Player::P1;
    let mut game = scenario(player);
    let drone = put(&mut game, player, "Kavaron Turbodrone", Zone::Battlefield);
    put(&mut game, player, "Molecular Modifier", Zone::Battlefield);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);

    activate(&mut game, drone, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    target(&mut game, Target::Card(friend));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(drone).tapped);
    assert_eq!(stats(&game, friend), (3, 3));
    assert!(game.object_view(friend).keywords.contains(Keywords::HASTE));

    until(&mut game, |game| game.state.step == Step::BeginCombat);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    target(&mut game, Target::Card(friend));
    settle(&mut game);
    assert_eq!(stats(&game, friend), (4, 3));
    assert!(
        game.object_view(friend)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
}

#[test]
fn melded_moxite_may_rummage_then_sacrifices_to_make_a_tapped_robot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Melded Moxite", Zone::Hand);
    let discard = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, artifact, None);
    resolve_top(&mut game);
    resolve_top(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.hand_count(player), 1);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 2);

    activate(&mut game, artifact, 0);
    pay(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    settle(&mut game);
    let token = game
        .observe(player)
        .objects
        .into_iter()
        .find(|view| view.zone == Zone::Battlefield && view.is_token() && view.has_subtype("Robot"))
        .unwrap();
    assert_eq!(
        (token.power, token.toughness, token.kinds, token.colors),
        (2, 2, 17, 0)
    );
    assert!(token.tapped);
}

#[test]
fn nebula_dragon_and_nova_hellkite_have_distinct_entry_damage_and_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let dragon = put(&mut game, player, "Nebula Dragon", Zone::Hand);
    cast(&mut game, dragon, None);
    resolve_top(&mut game);
    target(&mut game, Target::Player(player.other()));
    settle(&mut game);
    assert!(game.object_view(dragon).keywords.contains(Keywords::FLYING));
    assert!(game.object_view(dragon).dealt_damage_this_turn);
    assert_eq!(game.state.players[player.other().index()].life, 17);

    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let nova = put(&mut game, player, "Nova Hellkite", Zone::Hand);
    act(&mut game, Action::AnnounceWarp { card: nova });
    pay(&mut game);
    resolve_top(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    target(&mut game, Target::Card(enemy));
    settle(&mut game);
    let view = game.object_view(nova);
    assert!(view.cast_for_warp);
    assert!(
        view.keywords
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    assert_eq!(game.object_view(enemy).damage, 1);
}

#[test]
fn red_tiger_mechan_can_warp_with_two_mana_and_recast_later_at_printed_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tiger = put(&mut game, player, "Red Tiger Mechan", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: tiger });
    act(&mut game, Action::AnnounceWarp { card: tiger });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(tiger).keywords.contains(Keywords::HASTE));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let turn = game.state.turn;
    until(&mut game, |game| {
        game.object_view(tiger).zone == Zone::Exile
    });
    assert_eq!(game.state.turn, turn);
    assert!(!game.can_cast_exiled_card(player, tiger));
    until(&mut game, |game| {
        game.state.turn == turn + 2 && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [0, 0, 0, 4, 0, 0];
    act(&mut game, Action::AnnounceExile { card: tiger });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(!game.object_view(tiger).cast_for_warp);
}

#[test]
fn rig_for_war_can_give_an_opponents_creature_reach_first_strike_and_three_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Rig for War", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (5, 2));
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::REACH.union(Keywords::FIRST_STRIKE))
    );
}

#[test]
fn ruinous_rampage_can_damage_opponent_or_exile_all_small_artifacts() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Ruinous Rampage", Zone::Hand);
    let cheap = [
        put(&mut game, player, "Goldvein Pick", Zone::Battlefield),
        put(
            &mut game,
            player.other(),
            "Chrome Companion",
            Zone::Battlefield,
        ),
    ];
    let large = put(
        &mut game,
        player.other(),
        "Galvanizing Sawship",
        Zone::Battlefield,
    );
    let nonartifact = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut damage = game.clone();
    act(&mut damage, Action::ChooseMode { mode: 0 });
    pay(&mut damage);
    settle(&mut damage);
    assert_eq!(damage.state.players[player.other().index()].life, 17);
    assert!(
        cheap
            .iter()
            .all(|&id| damage.object_view(id).zone == Zone::Battlefield)
    );

    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    settle(&mut game);
    assert!(
        cheap
            .iter()
            .all(|&id| game.object_view(id).zone == Zone::Exile)
    );
    assert_eq!(game.object_view(large).zone, Zone::Battlefield);
    assert_eq!(game.object_view(nonartifact).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.other().index()].life, 20);
}

#[test]
fn slagdrill_scrapper_can_pay_another_artifact_or_land_for_one_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scrapper = put(&mut game, player, "Slagdrill Scrapper", Zone::Battlefield);
    let donors = [
        put(&mut game, player, "Forest", Zone::Battlefield),
        put(&mut game, player, "Goldvein Pick", Zone::Battlefield),
    ];
    activate(&mut game, scrapper, 0);
    let invalid = game.object_ref(scrapper);
    reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
    for donor in donors {
        let mut paid = game.clone();
        let object = paid.object_ref(donor);
        act(&mut paid, Action::ChooseAdditionalCost { object });
        pay(&mut paid);
        assert!(paid.object_view(scrapper).tapped);
        assert_eq!(paid.object_view(donor).zone, Zone::Graveyard);
        settle(&mut paid);
        assert_eq!(paid.hand_count(player), 1);
    }
}

#[test]
fn warmaker_gunship_counts_itself_on_entry_and_stations_at_six() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Hand);
    for _ in 0..2 {
        put(&mut game, player, "Goldvein Pick", Zone::Battlefield);
    }
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let donors = [
        put(&mut game, player, "Hill Giant", Zone::Battlefield),
        put(&mut game, player, "Hill Giant", Zone::Battlefield),
    ];
    cast(&mut game, ship, None);
    resolve_top(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(donors[0]),
        },
    );
    target(&mut game, Target::Card(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    for (donor, amount) in donors.into_iter().zip([3, 6]) {
        activate(&mut game, ship, 0);
        let object = game.object_ref(donor);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(ship, CounterKind::Charge), amount);
        assert_eq!(game.object_view(ship).is_creature(), amount == 6);
    }
    let view = game.object_view(ship);
    assert_eq!((view.power, view.toughness), (4, 3));
    assert!(view.has_subtype("Spacecraft"));
    assert!(view.keywords.contains(Keywords::FLYING));
}

#[test]
fn zookeeper_mechan_produces_red_mana_and_has_a_separate_paid_growth_activation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let keeper = put(&mut game, player, "Zookeeper Mechan", Zone::Battlefield);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let before = game.state.players[player.index()].mana;
    act(
        &mut game,
        Action::ActivateMana {
            card: keeper,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].mana[3], before[3] + 1);
    assert!(game.object_view(keeper).tapped);

    activate(&mut game, keeper, 1);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(ally));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (6, 2));
}
