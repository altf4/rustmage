// Full landfall, historical fourth-counter crossing and sacrifice-success reflexive.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Plan,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CounterThresholdCrossed {
                kind: CounterKind::Plan,
                threshold: 4,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSourceReflexive { ability: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Indestructible,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
