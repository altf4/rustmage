// Entry facts precede the lasting removal and haste effect on the actual entrant.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceBlankWithHaste],
    }],
    ..CardRules::EMPTY
};
