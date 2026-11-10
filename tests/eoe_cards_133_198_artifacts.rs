//! Actual EOE enchantment, artifact, and late red card behavior.
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

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn tokens(game: &Game, card: Card) -> Vec<CardId> {
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
const MUNITIONS: Card = Card::from_catalogue_id(5538);

#[test]
fn pain_for_all_uses_the_enchanted_creature_for_both_damage_abilities() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, player, "Pain for All", Zone::Hand);
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);

    act(&mut game, Action::AnnounceCast { card: aura });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    target(&mut game, Target::Card(own));
    pay(&mut game);
    resolve_top(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(own))
    );

    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    target(&mut game, Target::Player(player.other()));
    settle(&mut game);
    assert!(game.object_view(own).dealt_damage_this_turn);
    assert!(!game.object_view(aura).dealt_damage_this_turn);
    assert_eq!(game.state.players[player.other().index()].life, 17);

    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(own)));
    settle(&mut game);
    assert_eq!(game.object_view(own).damage, 2);
    assert_eq!(game.state.players[player.other().index()].life, 15);
}

#[test]
fn rust_harvester_exiles_an_own_artifact_as_cost_and_uses_post_counter_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let robot = put(&mut game, player, "Rust Harvester", Zone::Battlefield);
    let donor = put(&mut game, player, "Chrome Companion", Zone::Graveyard);
    let foreign = put(
        &mut game,
        player.other(),
        "Chrome Companion",
        Zone::Graveyard,
    );
    let nonartifact = put(&mut game, player, "Shock", Zone::Graveyard);
    assert!(game.object_view(robot).keywords.contains(Keywords::MENACE));

    activate(&mut game, robot);
    target(&mut game, Target::Player(player.other()));
    for invalid in [foreign, nonartifact] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert!(game.object_view(robot).tapped);
    assert_eq!(game.object_view(donor).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.counter_count(robot, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn systems_override_takes_untaps_hastes_and_then_removes_only_its_ten_charges() {
    let player = Player::P0;
    for spacecraft in [false, true] {
        let mut game = scenario(player);
        let name = if spacecraft {
            "Warmaker Gunship"
        } else {
            "Chrome Companion"
        };
        let stolen = put(&mut game, player.other(), name, Zone::Battlefield);
        game.state.objects[stolen as usize].tapped = true;
        game.add_counter(stolen, CounterKind::Charge, 2);
        let spell = put(&mut game, player, "Systems Override", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(stolen)));
        settle(&mut game);
        let view = game.object_view(stolen);
        assert_eq!(view.controller, player);
        assert!(!view.tapped);
        assert!(view.keywords.contains(Keywords::HASTE));
        assert_eq!(
            game.counter_count(stolen, CounterKind::Charge),
            if spacecraft { 12 } else { 2 }
        );

        until(&mut game, |game| game.state.step == Step::End);
        settle(&mut game);
        assert_eq!(game.object_view(stolen).controller, player);
        assert_eq!(game.counter_count(stolen, CounterKind::Charge), 2);
        let turn = game.state.turn;
        until(&mut game, |game| game.state.turn == turn + 1);
        let view = game.object_view(stolen);
        assert_eq!(view.controller, player.other());
        assert!(!view.keywords.contains(Keywords::HASTE));
    }
}

#[test]
fn tannuk_grants_only_other_controlled_creatures_haste_and_qualifying_hand_cards_warp() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tannuk = put(&mut game, player, "Tannuk, Steadfast Second", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let red = put(&mut game, player, "Kavaron Skywarden", Zone::Hand);
    let artifact = put(&mut game, player, "Goldvein Pick", Zone::Hand);
    let green = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);

    cast(&mut game, tannuk, None);
    settle(&mut game);
    assert!(game.object_view(own).keywords.contains(Keywords::HASTE));
    for id in [tannuk, foreign] {
        assert!(!game.object_view(id).keywords.contains(Keywords::HASTE));
    }
    let route = |card| Action::AnnounceGrantedWarp {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
        cost: ManaCost {
            generic: 2,
            colored: [0, 0, 0, 1, 0, 0],
        },
    };
    assert!(game.legal_actions().contains(&route(red)));
    reject(&mut game, route(green));
    reject(&mut game, route(instant));
    act(&mut game, route(artifact));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(artifact).cast_for_warp);
    until(&mut game, |game| game.state.step == Step::End);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn terminal_velocity_optionally_grants_haste_and_two_triggers_to_the_entered_permanent() {
    let player = Player::P1;
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Chrome Companion", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Terminal Velocity", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    let object = game.object_ref(recipient);
    assert_eq!(
        game.observe(player).effect_choice.unwrap().candidates,
        [object]
    );
    assert!(game.observe(player.other()).effect_choice.is_none());
    let wrong = game.object_ref(land);
    reject(&mut game, Action::PutHandPermanent { object: wrong });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineHandEntry);
    assert_eq!(declined.object_view(recipient).zone, Zone::Hand);
    act(&mut game, Action::PutHandPermanent { object });
    assert!(
        game.object_view(recipient)
            .keywords
            .contains(Keywords::HASTE)
    );

    until(&mut game, |game| game.state.step == Step::End);
    assert_eq!(game.object_view(recipient).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).damage, 2);
}

#[test]
fn terrapact_intimidator_gives_the_targeted_opponent_both_choices() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kavu = put(&mut game, player, "Terrapact Intimidator", Zone::Hand);
    cast(&mut game, kavu, None);
    resolve_top(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target(&mut game, Target::Player(player.other()));
    resolve_top(&mut game);
    assert_eq!(game.actor(), Some(player.other()));

    let mut accepted = game.clone();
    act(&mut accepted, Action::ChooseOptional { accept: true });
    let made = tokens(&accepted, LANDER);
    assert_eq!(made.len(), 2);
    assert!(
        made.iter()
            .all(|&id| accepted.object_view(id).controller == player)
    );
    assert_eq!(accepted.counter_count(kavu, CounterKind::PlusOne), 0);
    act(&mut game, Action::ChooseOptional { accept: false });
    assert!(tokens(&game, LANDER).is_empty());
    assert_eq!(game.counter_count(kavu, CounterKind::PlusOne), 2);
}

#[test]
fn territorial_bruntar_landfall_exiles_to_first_nonland_and_grants_only_its_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bruntar = put(&mut game, player, "Territorial Bruntar", Zone::Battlefield);
    let below = put(&mut game, player, "Divination", Zone::Library);
    let hit = put(&mut game, player, "Shock", Zone::Library);
    let skipped = put(&mut game, player, "Island", Zone::Library);
    let played = put(&mut game, player, "Forest", Zone::Hand);
    assert!(game.object_view(bruntar).keywords.contains(Keywords::REACH));
    act(&mut game, Action::PlayLand { card: played });
    settle(&mut game);
    assert_eq!(game.object_view(below).zone, Zone::Library);
    assert_eq!(game.object_view(skipped).zone, Zone::Exile);
    assert_eq!(game.object_view(hit).zone, Zone::Exile);
    assert!(!game.can_play_exiled_land(player, skipped));
    assert!(game.can_cast_exiled_card(player, hit));
    act(&mut game, Action::AnnounceExile { card: hit });
    target(&mut game, Target::Player(player.other()));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn vaultguard_trooper_allows_discarding_an_empty_hand_only_if_two_creatures_are_tapped() {
    let player = Player::P0;
    for enough in [false, true] {
        let mut game = scenario(player);
        let trooper = put(&mut game, player, "Vaultguard Trooper", Zone::Battlefield);
        let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.objects[trooper as usize].tapped = true;
        game.state.objects[other as usize].tapped = enough;
        until(&mut game, |game| game.state.step == Step::End);
        if enough {
            resolve_top(&mut game);
            assert_eq!(game.hand_count(player), 0);
            let mut declined = game.clone();
            act(&mut declined, Action::ChooseOptional { accept: false });
            assert_eq!(declined.hand_count(player), 0);
            act(&mut game, Action::ChooseOptional { accept: true });
            assert_eq!(game.hand_count(player), 2);
        } else {
            assert!(game.state.stack.is_empty());
            assert_eq!(game.hand_count(player), 0);
        }
    }
}

#[test]
fn weapons_manufacturing_makes_munitions_only_for_nontoken_artifacts_and_exile_triggers_damage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let weapons = put(&mut game, player, "Weapons Manufacturing", Zone::Hand);
    cast(&mut game, weapons, None);
    settle(&mut game);
    assert!(tokens(&game, MUNITIONS).is_empty());

    let token_spell = put(&mut game, player, "Lithobraking", Zone::Hand);
    cast(&mut game, token_spell, None);
    resolve_top(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    assert_eq!(tokens(&game, LANDER).len(), 1);
    assert!(tokens(&game, MUNITIONS).is_empty());

    let card = put(&mut game, player, "Goldvein Pick", Zone::Hand);
    cast(&mut game, card, None);
    settle(&mut game);
    let [munition] = tokens(&game, MUNITIONS).try_into().unwrap();
    let view = game.object_view(munition);
    assert_eq!((view.name, view.kinds, view.colors), ("Munitions", 16, 0));
    let exile = put(&mut game, player, "Exorcise", Zone::Hand);
    cast(&mut game, exile, Some(Target::Card(munition)));
    resolve_top(&mut game);
    assert_ne!(game.object_view(munition).zone, Zone::Battlefield);
    target(&mut game, Target::Player(player.other()));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn weftstalker_ardent_warps_and_sees_creatures_and_artifacts_once_per_entrant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let drix = put(&mut game, player, "Weftstalker Ardent", Zone::Hand);
    act(&mut game, Action::AnnounceWarp { card: drix });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(drix).cast_for_warp);
    assert_eq!(game.state.players[player.other().index()].life, 20);

    for name in ["Chrome Companion", "Bear Cub"] {
        let permanent = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, permanent, None);
        settle(&mut game);
    }
    assert_eq!(game.state.players[player.other().index()].life, 18);
    let spell = put(&mut game, player, "Lithobraking", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 17);
}

#[test]
fn atmospheric_greenhouse_counters_your_creatures_and_animates_only_at_eight_charges() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let ship = put(&mut game, player, "Atmospheric Greenhouse", Zone::Hand);
    cast(&mut game, ship, None);
    settle(&mut game);
    for creature in [first, second] {
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    }
    assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 0);
    assert!(!game.object_view(ship).is_creature());
    for (donor, charge) in [(first, 4), (second, 8)] {
        activate(&mut game, ship);
        let object = game.object_ref(donor);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(ship, CounterKind::Charge), charge);
        assert_eq!(game.object_view(ship).is_creature(), charge == 8);
    }
    let view = game.object_view(ship);
    assert_eq!((view.power, view.toughness), (5, 4));
    assert!(
        view.keywords
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
}
