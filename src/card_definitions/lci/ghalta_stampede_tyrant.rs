pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(12, 12),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DeployControllerHandCreatures],
    }],
    ..CardRules::EMPTY
};
