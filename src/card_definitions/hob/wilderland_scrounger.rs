// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 6),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCreatureCounters {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
