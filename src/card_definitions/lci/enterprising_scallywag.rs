pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
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
        program: &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
