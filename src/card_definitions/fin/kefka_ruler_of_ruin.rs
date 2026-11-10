pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::LifeLost {
                relation: CastRelation::Opponent,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::LifeEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};
