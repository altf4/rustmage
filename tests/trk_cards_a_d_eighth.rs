//! Typed Star Trek A–D card interactions after the shared mechanic audit.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}

fn finish(game: &mut Game) {
    for _ in 0..48 {
        if matches!(game.state.pending, Pending::Priority | Pending::Blockers)
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::Discard { .. }
            )
        });
    }
    panic!("resolution budget");
}

#[test]
fn anthropological_mission_discards_until_the_third_quest_counter() {
    for (prior, expected_hand) in [(0, 0), (2, 1)] {
        let mut game = scenario(Player::P0);
        let mission = owned(&mut game, "Anthropological Mission", Zone::Battlefield);
        game.set_counters(mission, Counters::of(CounterKind::Quest, prior));
        let spell = put(&mut game, Player::P0, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(Player::P1)));
        finish(&mut game);
        assert_eq!(game.counter_count(mission, CounterKind::Quest), prior + 1);
        assert_eq!(game.hand_count(Player::P0), expected_hand);
    }
}

#[test]
fn astute_terraformer_reduces_only_generic_cost_and_activates_once() {
    let mut game = scenario(Player::P0);
    let source = owned(&mut game, "Astute Terraformer", Zone::Battlefield);
    for name in ["Forest", "Island", "Mountain"] {
        put(&mut game, Player::P0, name, Zone::Battlefield);
    }
    let mana_before: u32 = game.state.players[0].mana.iter().copied().sum();
    let activate = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    act(&mut game, activate.clone());
    pay(&mut game);
    finish(&mut game);
    let mana_after: u32 = game.state.players[0].mana.iter().copied().sum();
    assert_eq!(mana_before - mana_after, 3);
    assert_eq!(stats(&game, source), (4, 4));
    reject(&mut game, activate);
}

#[test]
fn beckett_mariner_promotes_on_tap_then_cashes_in_three() {
    for (prior, expected_counters, expected_life) in [(0, 1, 20), (2, 0, 18)] {
        let mut game = scenario(Player::P0);
        let beckett = owned(
            &mut game,
            "Beckett Mariner, Impetuous Ensign",
            Zone::Battlefield,
        );
        game.set_counters(beckett, Counters::of(CounterKind::Promotion, prior));
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: beckett });
        act(&mut game, Action::FinishAttackers);
        finish(&mut game);
        assert_eq!(
            game.counter_count(beckett, CounterKind::Promotion),
            expected_counters
        );
        assert_eq!(game.state.players[1].life, expected_life);
    }
}
