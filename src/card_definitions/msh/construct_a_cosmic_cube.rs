// Direct whole Plan program; player-control effects outlive their source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: Some(2),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CreateTokens {
                    token: Card(5312),
                    amount: 1,
                    tapped: false,
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
                threshold: 7,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSourceReflexive { ability: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::ControlTargetPlayerNextTurn],
        },
    ],
    ..CardRules::EMPTY
};
