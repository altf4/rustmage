pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutDepartedSourceOnLibraryBottom],
    }],
    ..CardRules::EMPTY
};
