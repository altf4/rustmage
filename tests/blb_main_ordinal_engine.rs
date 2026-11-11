//! A postcombat main phase is not necessarily the second main phase.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SECOND_MAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::SecondMainPhase,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Main2,
                relation: CastRelation::You,
                guard: StepGuard::OpponentLostLife,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn begin_main(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.give_priority();
    game.prepare_triggers();
    settle(game);
}

#[test]
fn second_main_check_uses_actual_ordinal_even_if_the_phase_repeats() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SECOND_MAIN)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        begin_main(&mut game, Step::Main1);
        let mut late = game.clone();

        game.gain_life(player.other(), 3);
        game.lose_life(player.other(), 1);
        begin_main(&mut game, Step::Main2);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        begin_main(&mut game, Step::Main2);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

        begin_main(&mut late, Step::Main2);
        late.lose_life(player.other(), 1);
        begin_main(&mut late, Step::Main2);
        assert_eq!(late.state.players[player.index()].cards_drawn_this_turn, 0);
    }
}
