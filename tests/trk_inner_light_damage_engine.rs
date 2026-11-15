//! A delayed creature-to-player hit retains its exact damage amount.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DELAYED_DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DelayTargetEvent {
            event: DelayedTargetEvent::CombatDamageToPlayer, ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn delayed_player_damage_remembers_amount_for_both_players_and_owned_restore() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DELAYED_DRAW)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        settle(&mut game);
        game.state.objects[creature as usize].summoning_sick = false;
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        let action = Action::Attack { card: creature };
        let revision = game.revision();
        let checkpoint = game.snapshot();
        let index = game.legal_actions().iter().position(|a| *a == action).unwrap();
        let mut indexed = game.clone();
        act(&mut game, action.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.apply_index(player, revision, index), Err(Error::StaleRevision));
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        act(&mut game, Action::FinishAttackers);
        for _ in 0..96 {
            if game.state.step == Step::EndCombat
                && game.state.pending == Pending::Priority
                && game.state.stack.is_empty()
                && game.state.pending_triggers.is_empty()
                && game.state.resolving.is_none() { break; }
            choose(&mut game, |a| matches!(a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
                | Action::AssignDamage { target: Target::Player(_), .. }
            ));
        }
        assert_eq!(game.state.players[player.other().index()].life, 18);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    }
}
