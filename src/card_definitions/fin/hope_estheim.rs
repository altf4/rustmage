pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Mill(
            PlayerSet::Opponent,
            MillAmount::Amount(AmountValue::LifeGainedThisTurn),
        )],
    }],
    ..CardRules::EMPTY
};
