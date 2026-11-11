pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledAuraAttachesToOpponentNonland,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainControlWhileEventAuraAttached],
    }],
    ..CardRules::EMPTY
};
