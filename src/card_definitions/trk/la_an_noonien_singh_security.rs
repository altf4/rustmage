pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Threshold {
                value: StepValue::TotalCreaturePower,
                minimum: Some(8),
                maximum: None,
            },
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BLOCK,
        }],
    }],
    ..CardRules::EMPTY
};
