pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeBecomesBlocked("Rat"),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Entering,
            power: 2,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
