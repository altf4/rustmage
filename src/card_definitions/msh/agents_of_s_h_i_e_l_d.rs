// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Entering,
            power: 1,
            toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
