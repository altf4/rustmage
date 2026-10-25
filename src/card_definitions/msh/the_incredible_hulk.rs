// Enrage observes actual positive damage, then tests exact-source attacking at resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(8, 8),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamaged,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::SourceAttacking,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::UntapSource,
            Instruction::AdditionalCombat,
        ],
    }],
    ..CardRules::EMPTY
};
