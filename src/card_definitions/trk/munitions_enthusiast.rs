pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: one(TargetRule::Any),
        program: &[Instruction::Damage(1)],
    }],
    ..CardRules::EMPTY
};
