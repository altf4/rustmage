pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::EnsureGoblinArmy, Instruction::AmassGoblins(1)],
    }],
    keywords: Keywords::HASTE,
    ..CardRules::EMPTY
};
