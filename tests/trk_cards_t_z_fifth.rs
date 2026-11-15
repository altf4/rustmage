//! Enterprise-D's real Station ability and first-charge event.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn station(game: &mut Game, ship: CardId, donor: CardId) {
    act(game, Action::AnnounceActivate {
        card: ship, ability: 0, counter_payment: Counters::default(),
    });
    act(game, Action::ChooseAdditionalCost { object: game.object_ref(donor) });
    pay(game);
    settle(game);
}

#[test]
fn enterprise_d_exiles_once_on_first_charge_event_and_animates_at_seven() {
    let mut game = scenario(Player::P0);
    let ship = put(&mut game, Player::P0, "U.S.S. Enterprise-D, Galaxy-Class", Zone::Battlefield);
    let donor_a = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let donor_b = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let donor_c = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, Player::P0, "Forest", Zone::Library);
    let second = put(&mut game, Player::P0, "Island", Zone::Library);
    let third = put(&mut game, Player::P0, "Mountain", Zone::Library);
    assert_eq!(game.object_view(ship).kinds & 1, 0);
    station(&mut game, ship, donor_a);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 2);
    assert_eq!(game.state.exile_permissions.len(), 1);
    station(&mut game, ship, donor_b);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 4);
    assert_eq!(game.state.exile_permissions.len(), 1);
    station(&mut game, ship, donor_c);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 7);
    assert_eq!(game.state.exile_permissions.len(), 1);
    assert_eq!(game.object_view(ship).kinds & 1, 1);
    assert_eq!(stats(&game, ship), (4, 5));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING.union(Keywords::VIGILANCE)));
    assert_eq!([first, second, third].into_iter().filter(|&id| game.object_view(id).zone == Zone::Exile).count(), 1);
}

fn choose_trigger_mode(game: &mut Game, mode: u16, expected: usize) {
    for _ in 0..24 {
        if game.legal_actions().contains(&Action::ChooseMode { mode }) {
            let choices = game.legal_actions().iter().filter(|a| matches!(a, Action::ChooseMode { .. })).count();
            assert_eq!(choices, expected, "pairs with legal targets");
            act(game, Action::ChooseMode { mode });
            return;
        }
        choose(game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Wesley trigger mode not offered");
}

#[test]
fn wesley_chooses_exactly_two_modes_with_separate_targets() {
    let mut game = scenario(Player::P0);
    let small = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let large = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let wesley = put(&mut game, Player::P0, "Wesley Crusher, Starfleet Cadet", Zone::Hand);
    assert!(game.object_view(wesley).keywords.contains(Keywords::FLASH));
    cast(&mut game, wesley, None);
    choose_trigger_mode(&mut game, 3, 3);
    act(&mut game, Action::ChooseTarget { target: Target::Card(small) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(large) });
    if game.legal_actions().contains(&Action::FinishStackTargets) { act(&mut game, Action::FinishStackTargets); }
    settle(&mut game);
    assert_eq!(game.object_view(small).zone, Zone::Hand);
    assert!(game.object_view(large).keywords.contains(Keywords::CANT_BE_BLOCKED));

    let mut draw = scenario(Player::P0);
    let target = put(&mut draw, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut draw, Player::P1, "Forest", Zone::Library);
    let second = put(&mut draw, Player::P1, "Island", Zone::Library);
    let wesley = put(&mut draw, Player::P0, "Wesley Crusher, Starfleet Cadet", Zone::Hand);
    cast(&mut draw, wesley, None);
    choose_trigger_mode(&mut draw, 5, 3);
    act(&mut draw, Action::ChooseTarget { target: Target::Card(target) });
    act(&mut draw, Action::ChooseTarget { target: Target::Player(Player::P1) });
    if draw.legal_actions().contains(&Action::FinishStackTargets) { act(&mut draw, Action::FinishStackTargets); }
    while !matches!(draw.state.pending, Pending::EffectDiscard { .. }) {
        choose(&mut draw, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    assert_eq!(draw.state.players[1].cards_drawn_this_turn, 2);
    assert!(draw.object_view(target).keywords.contains(Keywords::CANT_BE_BLOCKED));
    act(&mut draw, Action::Discard { card: first });
    act(&mut draw, Action::Discard { card: second });
    settle(&mut draw);
    assert_eq!(draw.object_view(first).zone, Zone::Graveyard);
    assert_eq!(draw.object_view(second).zone, Zone::Graveyard);

    let mut counter = scenario(Player::P0);
    let small = put(&mut counter, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let pending = put(&mut counter, Player::P0, "Shock", Zone::Hand);
    cast(&mut counter, pending, Some(Target::Player(Player::P1)));
    let wesley = put(&mut counter, Player::P0, "Wesley Crusher, Starfleet Cadet", Zone::Hand);
    cast(&mut counter, wesley, None);
    choose_trigger_mode(&mut counter, 0, 6);
    act(&mut counter, Action::ChooseTarget { target: Target::Spell(pending) });
    act(&mut counter, Action::ChooseTarget { target: Target::Card(small) });
    if counter.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut counter, Action::FinishStackTargets);
    }
    for _ in 0..40 {
        if counter.legal_actions().contains(&Action::DeclineResolutionMana) { break; }
        choose(&mut counter, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut counter, Action::DeclineResolutionMana);
    settle(&mut counter);
    assert_eq!(counter.object_view(pending).zone, Zone::Graveyard);
    assert_eq!(counter.object_view(small).zone, Zone::Hand);
    assert_eq!(counter.state.players[1].life, 20);
}
