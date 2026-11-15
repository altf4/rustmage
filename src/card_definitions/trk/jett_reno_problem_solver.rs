// Oracle and official TRK release notes: power is checked when the tap trigger
// resolves; facing one dilemma adds one +1/+1 counter.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfAmountAtLeast {
                    value: AmountValue::SourcePower,
                    minimum: 4,
                    skip: 2,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::Skip(1),
                Instruction::Surveil(1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::FaceDilemma(CastRelation::You),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
