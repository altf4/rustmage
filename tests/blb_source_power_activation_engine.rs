//! A live activation power restriction applies to each supplied mana activation.

use crate::support::*;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::SourcePowerAtLeast(3),
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 1, 0, 0, 0, 0,
        ]))],
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn live_power_changes_mana_activation_without_hiding_unfunded_spell_attempts() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opt = put(&mut game, player, "Opt", Zone::Hand);
    let announce = Action::AnnounceCast { card: opt };
    let is_source_mana = |action: &Action| {
        matches!(action,
            Action::ActivateMana { card, ability: 0, .. }
            | Action::AnnounceMana { card, ability: 0, .. } if *card == source
        )
    };
    game.state.players[player.index()].mana = [0; 6];
    assert_eq!(stats(&game, source), (2, 2));
    assert!(!game.legal_actions().iter().any(is_source_mana));
    fail_unfunded_attempt(&mut game, announce.clone(), player);

    let pump = put(&mut game, player, "Shore Up", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, pump, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.legal_actions().contains(&announce));
    let mana_action = game
        .legal_actions()
        .into_iter()
        .find(is_source_mana)
        .unwrap();
    let mut powered = game.clone();
    act(&mut powered, mana_action.clone());
    if matches!(powered.state.pending, Pending::PayCosts { .. }) {
        pay(&mut powered);
    }
    assert_eq!(
        powered.state.players[player.index()].mana,
        [0, 1, 0, 0, 0, 0]
    );
    assert!(powered.object_view(source).tapped);

    game.finish_cleanup();
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    assert_eq!(stats(&game, source), (2, 2));
    reject(&mut game, mana_action);
    fail_unfunded_attempt(&mut game, announce, player);
}

fn fail_unfunded_attempt(game: &mut Game, announce: Action, player: Player) {
    assert!(game.legal_actions().contains(&announce));
    let source = match announce {
        Action::AnnounceCast { card } => game.object_ref(card),
        _ => unreachable!(),
    };
    act(game, announce);
    assert!(game.mana_payment_domain().is_none());
    act(game, Action::EndManaWindow);
    act(game, Action::SubmitAttempt);
    let outcome = game.last_attempt_outcome_view(player).unwrap();
    assert_eq!(outcome.outcome, AttemptOutcome::Failed);
    assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
    assert_eq!(game.object_ref(source.card), source);
    assert_eq!(game.object_view(source.card).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.state.stack.is_empty());
}
