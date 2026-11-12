pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Discard(PlayerSet::Controller, 1),
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                    value: StepValue::Hand,
                    minimum: None,
                    maximum: Some(0),
                }),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::SourceSolved,
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::Upkeep,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardHand(PlayerSet::Controller),
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
    ],
    ..CardRules::EMPTY
};
