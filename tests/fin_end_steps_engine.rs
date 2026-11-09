mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const END_STEPS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::AdditionalEndStepIfFirst,
            Instruction::AdditionalEndStepIfFirst,
        ],
    }],
    ..CardRules::EMPTY
};

const DELAYED_GAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ScheduleStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
    }],
    ..CardRules::EMPTY
};

fn pass_step(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn settle_end_triggers(game: &mut Game) {
    for _ in 0..20 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("end-step triggers did not settle");
}

#[test]
fn additional_end_steps_restart_triggers_without_cleanup_and_branch_independently() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Bear Cub").unwrap(), &END_STEPS),
        (Card::from_name("Opt").unwrap(), &DELAYED_GAIN),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    game.state.step = Step::Main2;
    game.state.objects[observer as usize].damage = 1;
    let initial_life = game.state.players[player.index()].life;
    let turn = game.state.turn;

    pass_step(&mut game);
    assert_eq!(game.state.step, Step::End);
    settle_end_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, initial_life + 1);
    assert_eq!(
        game.observe(player.other()).additional_end_steps,
        Some(AdditionalEndStepsView {
            pending: 2,
            current_is_additional: false,
        })
    );
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, initial_life + 1);

    act(&mut game, Action::Pass);
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player.other(), indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), before);
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    act(&mut indexed, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());

    settle_end_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, initial_life + 7);
    assert_eq!(game.state.turn, turn);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.object_view(observer).damage, 1);
    assert_eq!(
        game.observe(player.other()).additional_end_steps,
        Some(AdditionalEndStepsView {
            pending: 1,
            current_is_additional: true,
        })
    );

    pass_step(&mut game);
    settle_end_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, initial_life + 8);
    assert_eq!(game.object_view(observer).damage, 1);
    pass_step(&mut game);
    assert_eq!(game.state.turn, turn + 1);
    assert_eq!(game.object_view(observer).damage, 0);
    assert!(game.observe(player).additional_end_steps.is_none());
}
