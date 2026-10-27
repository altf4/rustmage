pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::EnsureGoblinArmy, Instruction::AmassGoblins(2)],
    }],
    ..CardRules::EMPTY
};
