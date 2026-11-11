pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::DiscardAnyDraw {
                players: PlayerSet::Controller,
                bonus: 1,
                required_all: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: StatValue::DescendedThisTurn,
                    minimum: Some(1),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Bore,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::SourceCounterQuantity(CounterKind::Bore),
                    minimum: 3,
                    skip: 2,
                },
                Instruction::RemoveAllSourceCounters(CounterKind::Bore),
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
