// Direct typed behavior from the paired Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
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
        program: &[Instruction::Scry(1)],
    }],
    ..CardRules::EMPTY
};
