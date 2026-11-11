pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
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
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
