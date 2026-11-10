//! Paid casts and activations for the first authored EOE red cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..128 {
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

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn announce_ability(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn battlefield_tokens(game: &Game, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.card == card)
                .then_some(id as CardId)
        })
        .collect()
}

const LANDER: Card = Card::from_catalogue_id(5238);
const ROBOT: Card = Card::from_catalogue_id(5309);

#[test]
fn drill_too_deep_separates_controlled_spacecraft_and_planet_from_artifact_destruction() {
    let player = Player::P0;
    let mut game = scenario(player);
    let planet = Card::from_name("Kavaron, Memorial World").unwrap();
    let own = spawn(&mut game, player, planet, Zone::Battlefield);
    let enemy = spawn(&mut game, player.other(), planet, Zone::Battlefield);
    let spacecraft = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let artifact = put(
        &mut game,
        player.other(),
        "Chrome Companion",
        Zone::Battlefield,
    );

    for recipient in [own, spacecraft, artifact] {
        let card = put(&mut game, player, "Drill Too Deep", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card });
        act(
            &mut game,
            Action::ChooseMode {
                mode: u16::from(recipient == artifact),
            },
        );
        let illegal = if recipient == artifact { own } else { enemy };
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(illegal),
            },
        );
        choose_target(&mut game, Target::Card(recipient));
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(game.counter_count(own, CounterKind::Charge), 5);
    assert_eq!(game.counter_count(spacecraft, CounterKind::Charge), 5);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
}

#[test]
fn galvanizing_sawship_stations_into_a_six_five_flying_hasty_artifact_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let donor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let wrong = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let ship = put(&mut game, player, "Galvanizing Sawship", Zone::Hand);
    cast(&mut game, ship, None);
    settle(&mut game);
    assert!(!game.object_view(ship).is_creature());
    assert!(!game.object_view(ship).keywords.contains(Keywords::FLYING));

    announce_ability(&mut game, ship);
    let enemy = game.object_ref(wrong);
    reject(&mut game, Action::ChooseAdditionalCost { object: enemy });
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert!(game.object_view(donor).tapped);
    settle(&mut game);

    let view = game.object_view(ship);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    assert_eq!(view.kinds & 17, 17);
    assert_eq!((view.power, view.toughness), (6, 5));
    assert!(
        view.keywords
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    until(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: ship });
}

#[test]
fn invasive_maneuvers_counts_your_noncreature_spacecraft() {
    let player = Player::P0;
    for controls in [false, true] {
        let mut game = scenario(player);
        let controller = if controls { player } else { player.other() };
        let ship = put(&mut game, controller, "Warmaker Gunship", Zone::Battlefield);
        assert!(!game.object_view(ship).is_creature());
        let dragon = put(
            &mut game,
            player.other(),
            "Shivan Dragon",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Invasive Maneuvers", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(dragon)));
        settle(&mut game);
        let target = game.object_view(dragon);
        if controls {
            assert_eq!(target.zone, Zone::Graveyard);
        } else {
            assert_eq!(target.zone, Zone::Battlefield);
            assert_eq!(target.damage, 3);
        }
    }
}

#[test]
fn kav_landseeker_waits_for_its_controllers_next_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kavu = put(&mut game, player, "Kav Landseeker", Zone::Hand);
    cast(&mut game, kavu, None);
    settle(&mut game);
    assert!(game.object_view(kavu).keywords.contains(Keywords::MENACE));
    let [lander] = battlefield_tokens(&game, LANDER).try_into().unwrap();
    assert!(game.object_view(lander).has_subtype("Lander"));
    let turn = game.state.turn;
    until(&mut game, |game| game.state.turn == turn + 2);
    assert_eq!(game.object_view(lander).zone, Zone::Battlefield);
    until(&mut game, |game| game.state.step == Step::End);
    assert_eq!(game.object_view(lander).zone, Zone::Battlefield);
    settle(&mut game);
    assert_ne!(game.object_view(lander).zone, Zone::Battlefield);
}

#[test]
fn kavaron_harrier_can_pay_two_for_an_attacking_robot_with_end_combat_sacrifice() {
    let player = Player::P1;
    let mut parent = scenario(player);
    let harrier = put(&mut parent, player, "Kavaron Harrier", Zone::Battlefield);
    let mountains = [
        put(&mut parent, player, "Mountain", Zone::Battlefield),
        put(&mut parent, player, "Mountain", Zone::Battlefield),
    ];
    until(&mut parent, |game| game.state.pending == Pending::Attackers);
    act(&mut parent, Action::Attack { card: harrier });
    act(&mut parent, Action::FinishAttackers);
    resolve_top(&mut parent);

    let mut paid = parent.clone();
    for card in mountains {
        act(&mut paid, Action::TapMana { card });
    }
    choose(&mut paid, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let [robot] = battlefield_tokens(&paid, ROBOT).try_into().unwrap();
    let view = paid.object_view(robot);
    assert!(view.tapped && view.attacking);
    assert_eq!((view.kinds & 17, view.power, view.toughness), (17, 2, 2));
    until(&mut paid, |game| game.state.step == Step::Main2);
    assert_ne!(paid.object_view(robot).zone, Zone::Battlefield);
    assert_eq!(paid.state.players[player.other().index()].life, 16);

    act(&mut parent, Action::DeclineResolutionMana);
    assert!(battlefield_tokens(&parent, ROBOT).is_empty());
    until(&mut parent, |game| game.state.step == Step::Main2);
    assert_eq!(parent.state.players[player.other().index()].life, 18);
}

#[test]
fn kavaron_skywarden_needs_void_at_its_controllers_end_step() {
    let player = Player::P0;
    for departure in [false, true] {
        let mut game = scenario(player);
        let skywarden = put(&mut game, player, "Kavaron Skywarden", Zone::Battlefield);
        assert!(
            game.object_view(skywarden)
                .keywords
                .contains(Keywords::REACH)
        );
        if departure {
            let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
            let removal = put(&mut game, player, "Lightning Strike", Zone::Hand);
            cast(&mut game, removal, Some(Target::Card(enemy)));
            settle(&mut game);
            assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        }
        until(&mut game, |game| game.state.step == Step::End);
        settle(&mut game);
        assert_eq!(
            game.counter_count(skywarden, CounterKind::PlusOne),
            u64::from(departure)
        );
    }
}

#[test]
fn lithobraking_can_sacrifice_its_lander_then_damage_with_a_separate_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Lithobraking", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    let [lander] = battlefield_tokens(&game, LANDER).try_into().unwrap();

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(lander).zone, Zone::Battlefield);
    assert!(declined.state.stack.is_empty());

    let illegal = game.object_ref(own);
    reject(&mut game, Action::Sacrifice { object: illegal });
    let object = game.object_ref(lander);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_ne!(game.object_view(lander).zone, Zone::Battlefield);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.kind, StackKind::TriggeredAbility { ability: 0 });
    assert_eq!(trigger.source().unwrap().id, spell);
    for creature in [own, enemy] {
        assert_eq!(game.object_view(creature).damage, 0);
    }
    settle(&mut game);
    for creature in [own, enemy] {
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    }
}

#[test]
fn memorial_vault_exiles_one_more_than_the_paid_artifacts_mana_value() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vault = put(&mut game, player, "Memorial Vault", Zone::Battlefield);
    let donor = put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    let below = put(&mut game, player, "Island", Zone::Library);
    let spell = put(&mut game, player, "Shock", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let mountain = put(&mut game, player, "Mountain", Zone::Library);

    announce_ability(&mut game, vault);
    let source = game.object_ref(vault);
    reject(&mut game, Action::ChooseAdditionalCost { object: source });
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert!(game.object_view(vault).tapped);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(below).zone, Zone::Library);
    for exiled in [spell, forest, mountain] {
        assert_eq!(game.object_view(exiled).zone, Zone::Exile);
    }
    assert!(game.can_cast_exiled_card(player, spell));
    act(&mut game, Action::PlayLand { card: mountain });
    assert_eq!(game.object_view(mountain).zone, Zone::Battlefield);
    let turn = game.state.turn;
    until(&mut game, |game| game.state.turn == turn + 1);
    assert!(!game.can_cast_exiled_card(player, spell));
}

#[test]
fn orbital_plunge_makes_a_lander_only_when_damage_exceeds_lethal() {
    let player = Player::P0;
    for excess in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Shivan Dragon",
            Zone::Battlefield,
        );
        if !excess {
            game.add_counter(target, CounterKind::PlusOne, 1);
        }
        let spell = put(&mut game, player, "Orbital Plunge", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Graveyard);
        assert_eq!(battlefield_tokens(&game, LANDER).len(), usize::from(excess));
    }
}

#[test]
fn oreplate_pangolin_sees_another_artifact_token_and_can_decline_its_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let pangolin = put(&mut game, player, "Oreplate Pangolin", Zone::Hand);
    cast(&mut game, pangolin, None);
    settle(&mut game);
    assert_eq!(game.counter_count(pangolin, CounterKind::PlusOne), 0);

    let spell = put(&mut game, player, "Lithobraking", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(battlefield_tokens(&game, LANDER).len(), 1);
    resolve_top(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(decline.counter_count(pangolin, CounterKind::PlusOne), 0);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    assert_eq!(game.counter_count(pangolin, CounterKind::PlusOne), 1);
}

#[test]
fn plasma_bolt_reads_void_at_resolution() {
    let player = Player::P0;
    for late_departure in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Plasma Bolt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        if late_departure {
            let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
            let response = put(&mut game, player, "Lightning Strike", Zone::Hand);
            cast(&mut game, response, Some(Target::Card(enemy)));
            resolve_top(&mut game);
            assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.other().index()].life,
            if late_departure { 17 } else { 18 }
        );
    }
}

#[test]
fn possibility_technician_tracks_each_exiled_card_and_a_live_kavu() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Possibility Technician", Zone::Hand);
    let top = put(&mut game, player, "Mountain", Zone::Library);
    act(&mut game, Action::AnnounceWarp { card: own });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert!(game.object_view(own).cast_for_warp);
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: top })
    );

    let other_kavu = put(&mut game, player, "Kavaron Skywarden", Zone::Hand);
    let second = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, other_kavu, None);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    game.move_object(own, Zone::Graveyard);
    assert!(game.can_cast_exiled_card(player, second));
    game.move_object(other_kavu, Zone::Graveyard);
    assert!(!game.can_cast_exiled_card(player, second));

    let third = put(&mut game, player, "Kavaron Skywarden", Zone::Hand);
    cast(&mut game, third, None);
    settle(&mut game);
    assert!(game.can_cast_exiled_card(player, second));
    act(&mut game, Action::PlayLand { card: top });
}

#[test]
fn roving_actuator_checks_void_then_exiles_and_can_cast_an_exact_copy() {
    let player = Player::P1;
    let mut parent = scenario(player);
    let valid = put(&mut parent, player, "Shock", Zone::Graveyard);
    let expensive = put(&mut parent, player, "Divination", Zone::Graveyard);
    let foreign = put(&mut parent, player.other(), "Shock", Zone::Graveyard);
    let robot = put(&mut parent, player, "Roving Actuator", Zone::Hand);

    let mut dormant = parent.clone();
    cast(&mut dormant, robot, None);
    settle(&mut dormant);
    assert_eq!(dormant.object_view(valid).zone, Zone::Graveyard);

    let enemy = put(&mut parent, player.other(), "Hill Giant", Zone::Battlefield);
    let departure = put(&mut parent, player, "Lightning Strike", Zone::Hand);
    cast(&mut parent, departure, Some(Target::Card(enemy)));
    settle(&mut parent);
    cast(&mut parent, robot, None);
    resolve_top(&mut parent);
    for invalid in [expensive, foreign] {
        reject(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
    let mut zero = parent.clone();
    act(&mut zero, Action::FinishTargets);
    settle(&mut zero);
    assert_eq!(zero.object_view(valid).zone, Zone::Graveyard);

    choose_target(&mut parent, Target::Card(valid));
    resolve_top(&mut parent);
    assert_eq!(parent.object_view(valid).zone, Zone::Exile);
    let mut decline = parent.clone();
    act(&mut decline, Action::FinishResolutionCasts);
    assert_eq!(decline.object_view(valid).zone, Zone::Exile);
    choose(
        &mut parent,
        |action| matches!(action, Action::CastExiledDuringResolution { object, face: CastFace::Normal } if object.card != valid),
    );
    choose_target(&mut parent, Target::Player(player.other()));
    pay(&mut parent);
    if parent
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut parent, Action::FinishResolutionCasts);
    }
    settle(&mut parent);
    assert_eq!(parent.state.players[player.other().index()].life, 18);
}
