//! Independent printed T–Z land and creature abilities.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn finish(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.resolving.is_none() && game.state.pending_triggers.is_empty() {
            return;
        }
        choose(game, |action| matches!(action,
            Action::Pass | Action::PutTrigger { .. }
            | Action::ArrangeLibrary { destination: LibraryDestination::Top, .. }
        ));
    }
    panic!("resolution did not finish");
}

#[test]
fn tell_me_what_you_see_opponent_choice_and_dilemma_bounce_are_independent() {
    for opponent_accepts in [false, true] {
        let mut game = scenario(Player::P0);
        let enchantment = put(&mut game, Player::P0, "Tell Me What You See", Zone::Hand);
        cast(&mut game, enchantment, None);
        for _ in 0..24 {
            let actions = game.legal_actions();
            if actions.contains(&Action::ChooseTarget { target: Target::Player(Player::P1) }) {
                act(&mut game, Action::ChooseTarget { target: Target::Player(Player::P1) });
            } else if actions.contains(&Action::FinishStackTargets) {
                act(&mut game, Action::FinishStackTargets);
            } else if matches!(game.state.pending, Pending::EffectOptional { .. }) {
                break;
            } else {
                choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
        }
        assert_eq!(game.actor(), Some(Player::P1));
        act(&mut game, Action::ChooseOptional { accept: opponent_accepts });
        finish(&mut game);
        assert_eq!(game.state.players[0].cards_drawn_this_turn, u64::from(opponent_accepts));
        assert_eq!(game.state.players[1].life, if opponent_accepts { 20 } else { 16 });
        assert_eq!(game.state.players[0].scried_or_surveilled_this_turn, !opponent_accepts);
    }

    let mut game = scenario(Player::P0);
    let enchantment = put(&mut game, Player::P0, "Tell Me What You See", Zone::Battlefield);
    let grave = put(&mut game, Player::P1, "Shock", Zone::Graveyard);
    let dilemma = put(&mut game, Player::P0, "Test of Courage", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: dilemma });
    act(&mut game, Action::ChooseMode { mode: 0 });
    act(&mut game, Action::ChooseTarget { target: Target::Card(grave) });
    if game.legal_actions().contains(&Action::FinishTargets) { act(&mut game, Action::FinishTargets); }
    pay(&mut game);
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) { break; }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    assert_eq!(game.actor(), Some(Player::P0));
    act(&mut game, Action::ChooseOptional { accept: true });
    finish(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Hand);
}
const ADD_LORE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield, kinds: 0, relation: 0,
            other: false, nonland: false, tapped: false,
            min_mana: 0, max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets, kind: CounterKind::Lore, amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn naked_time_searches_basic_mountain_and_grants_entry_counters_only_this_turn() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ADD_LORE)]);
    let mut game = scenario(Player::P0);
    let mountain = put(&mut game, Player::P0, "Mountain", Zone::Library);
    let forest = put(&mut game, Player::P0, "Forest", Zone::Library);
    let saga = put(&mut game, Player::P0, "The Naked Time", Zone::Hand);
    cast(&mut game, saga, None);
    for _ in 0..20 {
        if game.legal_actions().contains(&Action::ChooseSearch { card: mountain }) { break; }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    reject(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::ChooseSearch { card: mountain });
    if game.legal_actions().contains(&Action::FinishSearch) { act(&mut game, Action::FinishSearch); }
    finish(&mut game);
    assert_eq!(game.object_view(mountain).zone, Zone::Battlefield);
    assert!(game.object_view(mountain).tapped);

    let lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, lore, Some(Target::Card(saga)));
    finish(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);

    let own = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut game, own, None);
    finish(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert!(game.object_view(own).keywords.contains(Keywords::HASTE));
    let opposing = put(&mut game, Player::P1, "Spectral Sailor", Zone::Hand);
    game.state.priority = Player::P1;
    cast(&mut game, opposing, None);
    finish(&mut game);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 0);

    game.state.priority = Player::P0;
    let next_lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, next_lore, Some(Target::Card(saga)));
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::Sacrifice { object: game.object_ref(mountain) }) {
            break;
        }
        choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    let mountain_ref = game.object_ref(mountain);
    act(&mut game, Action::Sacrifice { object: mountain_ref });
    finish(&mut game);
    assert_eq!(game.object_view(mountain).zone, Zone::Graveyard);
}

#[test]
fn measure_of_a_man_links_nontoken_exile_investigates_and_returns_to_owner() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ADD_LORE)]);
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P1, "Tricorder", Zone::Battlefield);
    let token = spawn(&mut game, Player::P1, Card(6008), Zone::Battlefield);
    let saga = put(&mut game, Player::P0, "The Measure of a Man", Zone::Hand);
    cast(&mut game, saga, None);
    for _ in 0..24 {
        let actions = game.legal_actions();
        if actions.contains(&Action::ChooseTarget { target: Target::Card(artifact) }) {
            reject(&mut game, Action::ChooseTarget { target: Target::Card(token) });
            act(&mut game, Action::ChooseTarget { target: Target::Card(artifact) });
        } else if actions.contains(&Action::FinishStackTargets) {
            act(&mut game, Action::FinishStackTargets);
        } else if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty() && game.state.resolving.is_none() {
            break;
        } else {
            choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
    }
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    for chapter in [2, 3] {
        let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(saga)));
        finish(&mut game);
        if chapter == 2 {
            assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
            assert_eq!(game.state.objects.iter().filter(|o| o.zone == Zone::Battlefield && o.card == Card(5296)).count(), 1);
        }
    }
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).controller, Player::P1);
    assert_eq!(game.state.players[0].life, 22);
}

#[test]
fn temporal_anomaly_pays_tap_and_sacrifice_then_selects_only_its_milled_permanent() {
    let mut game = scenario(Player::P0);
    let land = put(&mut game, Player::P0, "Temporal Anomaly", Zone::Battlefield);
    let old = put(&mut game, Player::P0, "Hill Giant", Zone::Graveyard);
    let artifact = put(&mut game, Player::P0, "Tricorder", Zone::Library);
    let nonpermanent = put(&mut game, Player::P0, "Shock", Zone::Library);
    let second = put(&mut game, Player::P0, "Forest", Zone::Library);
    let fourth = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let mut mana = game.clone();
    let action = mana.legal_actions().into_iter().find(|action|
        matches!(action, Action::ActivateMana { card, .. } if *card == land)).unwrap();
    act(&mut mana, action);
    assert!(mana.object_view(land).tapped);
    assert_eq!(mana.state.players[0].mana[5], 9);

    act(&mut game, Action::AnnounceActivate {
        card: land, ability: 1, counter_payment: Counters::default(),
    });
    pay(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    for _ in 0..20 {
        if matches!(game.state.pending, Pending::MilledSelection { .. }) { break; }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    assert!(matches!(game.state.pending, Pending::MilledSelection { .. }));
    let old_ref = game.object_ref(old);
    let nonpermanent_ref = game.object_ref(nonpermanent);
    reject(&mut game, Action::SelectMilled { object: old_ref });
    reject(&mut game, Action::SelectMilled { object: nonpermanent_ref });
    let selected = game.object_ref(artifact);
    act(&mut game, Action::SelectMilled { object: selected });
    finish(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.object_view(fourth).zone, Zone::Graveyard);
    let planet = game.state.objects.iter().enumerate().find_map(|(id, object)|
        (object.zone == Zone::Battlefield && object.card == Card(6009)).then_some(id as CardId)
    ).expect("New Planet token");
    assert!(game.object_view(planet).tapped);
}

#[test]
fn worf_taps_an_artifact_and_creature_then_doubles_live_power_on_attack() {
    let mut game = scenario(Player::P0);
    let worf = put(&mut game, Player::P0, "Worf, Honor-Bound Warrior", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    let donor = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceActivate {
        card: worf, ability: 0, counter_payment: Counters::default(),
    });
    let wrong = game.object_ref(enemy);
    reject(&mut game, Action::ChooseAdditionalCost { object: wrong });
    for card in [artifact, donor] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    pay(&mut game);
    finish(&mut game);
    assert_eq!(game.counter_count(worf, CounterKind::PlusOne), 1);
    assert!(game.object_view(artifact).tapped && game.object_view(donor).tapped);
    game.state.objects[worf as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: worf });
    act(&mut game, Action::FinishAttackers);
    finish(&mut game);
    assert_eq!(stats(&game, worf), (4, 2));
}
