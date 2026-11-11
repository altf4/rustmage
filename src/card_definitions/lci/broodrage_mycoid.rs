const FUNGUS: Card = Card::from_catalogue_id(5292);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
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
            token: FUNGUS,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
