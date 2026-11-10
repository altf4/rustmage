// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttackAtLeast {
                subtype: "Spider",
                minimum: 2,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SourceDealtDamageAtLeast(4),
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlayScaled {
                amount: AmountValue::Fixed(1),
                next_turn: true,
            }],
        },
    ],
    ..CardRules::EMPTY
};
