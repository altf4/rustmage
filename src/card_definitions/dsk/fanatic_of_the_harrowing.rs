pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscardThenDrawForOwnCount {
            players: PlayerSet::Each,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
