//! Complete T–Z Star Trek definitions exercised through the public card catalogue.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain(game: &mut Game, target: Option<CardId>) {
    for _ in 0..64 {
        let actions = game.legal_actions();
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        if let Some(target) = target
            && actions.contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        {
            act(game, Action::ChooseTarget { target: Target::Card(target) });
        } else if actions.contains(&Action::FinishStackTargets) {
            act(game, Action::FinishStackTargets);
        } else if let Some(action) = actions.into_iter().find(|action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        }) {
            act(game, action);
        } else {
            panic!("unhandled stack choice: {:?}", game.state.pending);
        }
    }
    panic!("stack resolution exceeded 64 actions");
}

#[test]
fn analyst_and_xenobotanist_use_real_tap_abilities() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let analyst = put(&mut game, player, "Tal Shiar Analyst", Zone::Battlefield);
        let botanist = put(&mut game, player, "Xenobotanist", Zone::Battlefield);
        for source in [analyst, botanist] {
            game.state.objects[source as usize].summoning_sick = false;
        }
        assert_eq!(game.object_view(analyst).power, 3);
        let life = game.state.players[player.other().index()].life;
        act(&mut game, Action::AnnounceActivate {
            card: analyst,
            ability: 0,
            counter_payment: Counters::default(),
        });
        pay(&mut game);
        drain(&mut game, None);
        assert!(game.state.objects[analyst as usize].tapped);
        assert_eq!(game.state.players[player.other().index()].life, life - 1);

        let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(analyst)));
        drain(&mut game, None);
        assert!(!game.state.objects[analyst as usize].tapped);

        let before = game.state.players[player.index()].mana;
        let mana = game.legal_actions().into_iter().find(|action| {
            matches!(action, Action::ActivateMana { card, .. } if *card == botanist)
        }).expect("Xenobotanist mana ability");
        act(&mut game, mana);
        assert!(game.state.objects[botanist as usize].tapped);
        assert!(game.state.players[player.index()].mana.iter().zip(before).any(|(after, before)| *after > before));
    }
}

#[test]
fn xindi_landfall_and_vulcan_entry_are_owner_scoped() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let xindi = put(&mut game, player, "Xindi Surveyors", Zone::Battlefield);
        let life = game.state.players[player.index()].life;
        let vulcan = put(&mut game, player, "Vulcan", Zone::Hand);
        act(&mut game, Action::PlayLand { card: vulcan });
        drain(&mut game, None);
        assert!(game.state.objects[vulcan as usize].tapped);
        assert_eq!(game.state.players[player.index()].life, life + 1);
        assert_eq!(game.counter_count(xindi, CounterKind::PlusOne), 1);
        let mut opponent_turn = scenario(player.other());
        let xindi = put(&mut opponent_turn, player, "Xindi Surveyors", Zone::Battlefield);
        let opposing = put(&mut opponent_turn, player.other(), "Forest", Zone::Hand);
        act(&mut opponent_turn, Action::PlayLand { card: opposing });
        drain(&mut opponent_turn, None);
        assert_eq!(opponent_turn.counter_count(xindi, CounterKind::PlusOne), 0);
    }
}

#[test]
fn tactical_officer_uses_total_power_at_attack_declaration() {
    let mut game = scenario(Player::P0);
    let officer = put(&mut game, Player::P0, "Tactical Officer", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    game.state.objects[officer as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: officer });
    act(&mut game, Action::FinishAttackers);
    while !game.state.stack.is_empty() || !game.state.pending_triggers.is_empty() {
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    assert_eq!(game.counter_count(officer, CounterKind::PlusOne), 1);
    assert!(!game.state.objects[officer as usize].tapped);
}

#[test]
fn vexing_drone_taps_only_an_opposing_creature() {
    let mut game = scenario(Player::P0);
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let drone = put(&mut game, Player::P0, "Vexing Drone", Zone::Hand);
    cast(&mut game, drone, None);
    drain(&mut game, Some(target));
    assert!(game.state.objects[target as usize].tapped);
    assert_eq!(game.object_view(drone).zone, Zone::Battlefield);
}
