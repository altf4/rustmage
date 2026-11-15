//! Actual Star Trek T–Z spells and tokens, separate from shared mechanic fixtures.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn modal(game: &mut Game, spell: CardId, mode: u16, target: Option<Target>) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn finish(game: &mut Game) {
    finish_target(game, None);
}

fn finish_target(game: &mut Game, target: Option<Target>) {
    for _ in 0..40 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.resolving.is_none() && game.state.pending_triggers.is_empty() {
            return;
        }
        let actions = game.legal_actions();
        let action = if let Some(target) = target
            && actions.contains(&Action::ChooseTarget { target }) {
            Action::ChooseTarget { target }
        } else if actions.contains(&Action::FinishStackTargets) {
            Action::FinishStackTargets
        } else {
            actions.into_iter().find(|action| matches!(action,
                Action::Pass | Action::PutTrigger { .. }
                | Action::ArrangeLibrary { .. }
                | Action::SelectLibrary { .. } | Action::FinishLibrarySelection
                | Action::FinishResolutionCasts
                | Action::ChooseOptional { accept: true }
            )).unwrap_or_else(|| panic!("unhandled {:?}", game.state.pending))
        };
        act(game, action);
    }
    panic!("resolution did not finish");
}

fn station(game: &mut Game, ship: CardId, donor: CardId) {
    act(game, Action::AnnounceActivate {
        card: ship, ability: 0, counter_payment: Counters::default(),
    });
    let object = game.object_ref(donor);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    finish(game);
}

#[test]
fn tenacious_drone_dies_into_the_printed_borg_token() {
    let mut game = scenario(Player::P0);
    let drone = put(&mut game, Player::P0, "Tenacious Drone", Zone::Battlefield);
    let shock = put(&mut game, Player::P1, "Shock", Zone::Hand);
    game.state.priority = Player::P1;
    cast(&mut game, shock, Some(Target::Card(drone)));
    finish(&mut game);
    assert_eq!(game.object_view(drone).zone, Zone::Graveyard);
    let borg = game.state.objects.iter().enumerate().find_map(|(id, object)|
        (object.zone == Zone::Battlefield && object.card == Card(6007)).then_some(id as CardId)
    ).expect("Borg token");
    assert_eq!(stats(&game, borg), (1, 1));
    assert!(game.object_view(borg).kinds & 16 != 0);
}

#[test]
fn test_of_conviction_both_modes_are_distinct() {
    let mut counters = scenario(Player::P0);
    let first = put(&mut counters, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut counters, Player::P0, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut counters, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut counters, Player::P0, "Test of Conviction", Zone::Hand);
    modal(&mut counters, spell, 0, None);
    finish(&mut counters);
    for creature in [first, second] { assert_eq!(counters.counter_count(creature, CounterKind::PlusOne), 1); }
    assert_eq!(counters.counter_count(enemy, CounterKind::PlusOne), 0);

    let mut draw = scenario(Player::P0);
    let spell = put(&mut draw, Player::P0, "Test of Conviction", Zone::Hand);
    modal(&mut draw, spell, 1, Some(Target::Player(Player::P1)));
    finish(&mut draw);
    assert_eq!(draw.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(draw.state.players[1].cards_drawn_this_turn, 1);
}

#[test]
fn test_of_courage_checks_graveyard_and_small_artifact_mode() {
    let mut grave = scenario(Player::P0);
    let card = put(&mut grave, Player::P1, "Shock", Zone::Graveyard);
    let spell = put(&mut grave, Player::P0, "Test of Courage", Zone::Hand);
    modal(&mut grave, spell, 0, Some(Target::Card(card)));
    finish(&mut grave);
    assert_eq!(grave.object_view(card).zone, Zone::Exile);
    assert_eq!(grave.state.players[0].life, 22);

    let mut permanent = scenario(Player::P0);
    let artifact = put(&mut permanent, Player::P1, "Tricorder", Zone::Battlefield);
    let creature = put(&mut permanent, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut permanent, Player::P0, "Test of Courage", Zone::Hand);
    act(&mut permanent, Action::AnnounceCast { card: spell });
    act(&mut permanent, Action::ChooseMode { mode: 1 });
    reject(&mut permanent, Action::ChooseTarget { target: Target::Card(creature) });
    act(&mut permanent, Action::ChooseTarget { target: Target::Card(artifact) });
    if permanent.legal_actions().contains(&Action::FinishTargets) { act(&mut permanent, Action::FinishTargets); }
    pay(&mut permanent);
    finish(&mut permanent);
    assert_eq!(permanent.object_view(artifact).zone, Zone::Graveyard);
}

#[test]
fn test_of_loyalty_respects_graveyard_owner_in_second_mode() {
    let mut exile = scenario(Player::P0);
    let enemy = put(&mut exile, Player::P1, "Shock", Zone::Graveyard);
    let spell = put(&mut exile, Player::P0, "Test of Loyalty", Zone::Hand);
    modal(&mut exile, spell, 0, Some(Target::Card(enemy)));
    finish(&mut exile);
    assert_eq!(exile.object_view(enemy).zone, Zone::Exile);
    assert_eq!(exile.state.players[0].cards_drawn_this_turn, 1);

    let mut return_card = scenario(Player::P0);
    let own = put(&mut return_card, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let enemy = put(&mut return_card, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let spell = put(&mut return_card, Player::P0, "Test of Loyalty", Zone::Hand);
    act(&mut return_card, Action::AnnounceCast { card: spell });
    act(&mut return_card, Action::ChooseMode { mode: 1 });
    reject(&mut return_card, Action::ChooseTarget { target: Target::Card(enemy) });
    act(&mut return_card, Action::ChooseTarget { target: Target::Card(own) });
    if return_card.legal_actions().contains(&Action::FinishTargets) { act(&mut return_card, Action::FinishTargets); }
    pay(&mut return_card);
    finish(&mut return_card);
    assert_eq!(return_card.object_view(own).zone, Zone::Hand);
    assert_eq!(return_card.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn test_of_priorities_blinks_only_own_creature_until_end_step() {
    let mut game = scenario(Player::P0);
    let own = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Test of Priorities", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, Action::ChooseTarget { target: Target::Card(opponent) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(own) });
    if game.legal_actions().contains(&Action::FinishTargets) { act(&mut game, Action::FinishTargets); }
    pay(&mut game);
    finish(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
    game.begin_step(Step::End);
    game.advance_replacements(false);
    game.give_priority();
    finish(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert_eq!(game.object_view(own).controller, Player::P0);
}

#[test]
fn transporter_malfunction_has_three_damage_and_printed_beam_cost() {
    let mut ordinary = scenario(Player::P0);
    let spell = put(&mut ordinary, Player::P0, "Transporter Malfunction", Zone::Hand);
    cast(&mut ordinary, spell, Some(Target::Player(Player::P1)));
    finish(&mut ordinary);
    assert_eq!(ordinary.state.players[1].life, 17);
    assert_eq!(ordinary.object_view(spell).zone, Zone::Graveyard);

    let mut beam = scenario(Player::P0);
    let spell = put(&mut beam, Player::P0, "Transporter Malfunction", Zone::Graveyard);
    let creature = put(&mut beam, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let cost = ManaCost { generic: 3, colored: [0, 0, 0, 3, 0, 0] };
    act(&mut beam, Action::AnnounceBeamMeUp {
        card: spell, face: CastFace::Normal, cost, flash_surcharge: false, provider: None,
    });
    act(&mut beam, Action::ChooseTarget { target: Target::Player(Player::P1) });
    if beam.legal_actions().contains(&Action::FinishTargets) { act(&mut beam, Action::FinishTargets); }
    act(&mut beam, Action::BeginOrderedCosts);
    let clauses: Vec<_> = beam.state.announcing.as_ref().unwrap().cast_costs.as_ref().unwrap()
        .clauses.iter().map(|row| row.clause).collect();
    for kind in [CastCostClause::Mana, clauses.into_iter().find(|kind| matches!(kind, CastCostClause::BeamMeUpReturn(_, _))).unwrap()] {
        let clause = beam.state.announcing.as_ref().unwrap().cast_costs.as_ref().unwrap()
            .clauses.iter().position(|row| row.clause == kind && !row.paid).unwrap() as u16;
        act(&mut beam, Action::ChooseCastCost { clause });
        if matches!(kind, CastCostClause::Mana) { pay(&mut beam); }
        else {
            let object = beam.object_ref(creature);
            act(&mut beam, Action::PayBeamMeUpReturn { object });
        }
    }
    finish(&mut beam);
    assert_eq!(beam.object_view(creature).zone, Zone::Hand);
    assert_eq!(beam.object_view(spell).zone, Zone::Exile);
    assert_eq!(beam.state.players[1].life, 17);
}

#[test]
fn cerritos_creates_two_officers_and_stations_at_six_charge() {
    let mut game = scenario(Player::P0);
    let ship = put(&mut game, Player::P0, "U.S.S. Cerritos, Following Up", Zone::Hand);
    cast(&mut game, ship, None);
    finish(&mut game);
    let officers: Vec<_> = game.state.objects.iter().enumerate().filter_map(|(id, object)|
        (object.zone == Zone::Battlefield && object.card == Card(6008)).then_some(id as CardId)
    ).collect();
    assert_eq!(officers.len(), 2);
    assert!(!game.object_view(ship).is_creature());
    let first = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceActivate {
        card: ship, ability: 0, counter_payment: Counters::default(),
    });
    let wrong = game.object_ref(enemy);
    reject(&mut game, Action::ChooseAdditionalCost { object: wrong });
    let donor = game.object_ref(first);
    act(&mut game, Action::ChooseAdditionalCost { object: donor });
    pay(&mut game);
    finish(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    assert!(!game.object_view(ship).is_creature());
    station(&mut game, ship, second);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 6);
    assert_eq!(stats(&game, ship), (4, 4));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn enterprise_a_creates_three_officers_and_station_grants_lifelink() {
    let mut game = scenario(Player::P0);
    let ship = put(&mut game, Player::P0, "U.S.S. Enterprise-A, Free Spirit", Zone::Hand);
    cast(&mut game, ship, None);
    finish(&mut game);
    assert_eq!(game.state.objects.iter().filter(|o|
        o.zone == Zone::Battlefield && o.card == Card(6008)).count(), 3);
    let first = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    station(&mut game, ship, first);
    station(&mut game, ship, second);
    assert_eq!(stats(&game, ship), (5, 5));
    let view = game.object_view(ship);
    assert!(view.keywords.contains(Keywords::FLYING));
    assert!(view.keywords.contains(Keywords::LIFELINK));
}

#[test]
fn excelsior_destroys_only_tapped_opposing_creature_and_stations_at_eight() {
    let mut game = scenario(Player::P0);
    let tapped = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let untapped = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let ship = put(&mut game, Player::P0, "U.S.S. Excelsior, Cutting Edge", Zone::Hand);
    cast(&mut game, ship, None);
    finish_target(&mut game, Some(Target::Card(tapped)));
    assert_eq!(game.object_view(tapped).zone, Zone::Graveyard);
    assert_eq!(game.object_view(untapped).zone, Zone::Battlefield);
    let first = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let third = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    for donor in [first, second, third] { station(&mut game, ship, donor); }
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 8);
    assert_eq!(stats(&game, ship), (4, 4));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn discovery_discovers_on_entry_and_attack_after_station() {
    let mut game = scenario(Player::P0);
    let hit = put(&mut game, Player::P0, "Shock", Zone::Library);
    let ship = put(&mut game, Player::P0, "U.S.S. Discovery, at Black Alert", Zone::Hand);
    cast(&mut game, ship, None);
    finish(&mut game);
    assert_eq!(game.object_view(hit).zone, Zone::Hand);
    let first = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    station(&mut game, ship, first);
    station(&mut game, ship, second);
    assert_eq!(stats(&game, ship), (5, 5));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
    let second_hit = put(&mut game, Player::P0, "Shock", Zone::Library);
    game.state.objects[ship as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: ship });
    act(&mut game, Action::FinishAttackers);
    finish(&mut game);
    assert_eq!(game.object_view(second_hit).zone, Zone::Hand);
}

#[test]
fn voyager_reveals_two_lands_into_battlefield_tapped() {
    let mut game = scenario(Player::P0);
    let first = put(&mut game, Player::P0, "Forest", Zone::Library);
    let gap = put(&mut game, Player::P0, "Shock", Zone::Library);
    let second = put(&mut game, Player::P0, "Island", Zone::Library);
    let ship = put(&mut game, Player::P0, "U.S.S. Voyager, Lone Wanderer", Zone::Hand);
    cast(&mut game, ship, None);
    finish(&mut game);
    for land in [first, second] {
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert!(game.object_view(land).tapped);
    }
    assert_eq!(game.object_view(gap).zone, Zone::Library);
    game.add_counter(ship, CounterKind::Charge, 8);
    assert_eq!(stats(&game, ship), (3, 3));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
    assert!(game.object_view(ship).keywords.contains(Keywords::VIGILANCE));
}

#[test]
fn reliant_returns_small_own_grave_creatures_with_finality_on_both_events() {
    let mut game = scenario(Player::P0);
    let first = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let ship = put(&mut game, Player::P0, "U.S.S. Reliant, From Hell's Heart", Zone::Hand);
    cast(&mut game, ship, None);
    finish_target(&mut game, Some(Target::Card(first)));
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(first, CounterKind::Finality), 1);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    game.add_counter(ship, CounterKind::Charge, 10);
    assert_eq!(stats(&game, ship), (4, 3));
    let second = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    game.state.objects[ship as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: ship });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..32 {
        if game.state.pending == (Pending::Targets { player: Player::P0 }) { break; }
        choose(&mut game, |action| matches!(action,
            Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
        ));
    }
    assert_eq!(game.state.pending, Pending::Targets { player: Player::P0 });
    finish_target(&mut game, Some(Target::Card(second)));
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(second, CounterKind::Finality), 1);
}
