pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: one(object(1 | 16)),
        program: &[Instruction::May { skip: 1 }, Instruction::TapOrUntap],
    }],
    ..CardRules::EMPTY
};
