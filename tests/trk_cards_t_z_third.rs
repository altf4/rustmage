//! Independent printed T–Z land and creature abilities.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn finish(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.resolving.is_none() && game.state.pending_triggers.is_empty() {
            return;
        }
        choose(game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("resolution did not finish");
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
