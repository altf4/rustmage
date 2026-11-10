pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBlocksOrBecomesBlockedByCreature,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamageToEventObject(1)],
    }],
    ..CardRules::EMPTY
};
