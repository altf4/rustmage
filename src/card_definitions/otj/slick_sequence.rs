pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Any),
        &[
            Instruction::Damage(2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::OtherSpellsCastThisTurn,
                minimum: 1,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
