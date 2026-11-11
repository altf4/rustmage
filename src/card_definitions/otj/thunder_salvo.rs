pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DamageScaled(AmountValue::Sum(&[
            AmountValue::Fixed(2),
            AmountValue::OtherSpellsCastThisTurn,
        ]))],
    ),
    ..CardRules::EMPTY
};
