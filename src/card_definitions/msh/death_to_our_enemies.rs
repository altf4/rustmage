// Directly authored from the complete pinned Oracle and XMage Plan implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: true,
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
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Any,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::DamageDivided { total: 7 }],
        },
    ],
    ..CardRules::EMPTY
};
