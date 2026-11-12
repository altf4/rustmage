//! A live activation power restriction participates in real and projected legality.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

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
fn live_power_changes_real_and_funded_activation_and_stale_actions_reject() {
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
    reject(&mut game, announce.clone());

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
    reject(&mut game, announce);
}
