pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::CardsDrawnThisTurn),
                minimum: 2,
                skip: 2,
            },
            Instruction::Damage(5),
            Instruction::Skip(1),
            Instruction::Damage(3),
        ],
    ),
    ..CardRules::EMPTY
};
