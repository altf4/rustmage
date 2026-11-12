pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: one(TargetRule::Player {
            opponent_only: false,
        }),
        program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(2))],
    }],
    ..CardRules::EMPTY
};
