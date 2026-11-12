// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5706),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OwnAttackerAbilityTriggered,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Quest,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::SourceCounterQuantity(CounterKind::Quest),
                    minimum: 4,
                    skip: 2,
                },
                Instruction::MayCopyRelatedTriggeredAbility { skip: 1 },
                Instruction::CopyRelatedTriggeredAbility,
            ],
        },
    ],
    ..CardRules::EMPTY
};
