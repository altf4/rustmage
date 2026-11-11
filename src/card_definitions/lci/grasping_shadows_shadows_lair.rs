pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Entering,
                keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Dread,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Dread),
                minimum: 3,
                skip: 1,
            },
            Instruction::TransformSource,
        ],
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
