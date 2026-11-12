pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Opponent, 1)],
    }],
    ..CardRules::EMPTY
};
