pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 1,
                max: 2,
            }),
            program: &[Instruction::DistributeCounters {
                total: 2,
                kind: CounterKind::PlusOne,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceUnsolved(&StepGuard::Threshold {
                    value: StepValue::TotalCreaturePower,
                    minimum: Some(8),
                    maximum: None,
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
                event: &TriggerCondition::AttackDeclared {
                    minimum: 1,
                    power_min: None,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            })),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
