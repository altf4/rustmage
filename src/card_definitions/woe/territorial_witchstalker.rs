pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEFENDER,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            },
            Instruction::AttackWithoutDefenderThisTurn,
        ],
    }],
    ..CardRules::EMPTY
};
