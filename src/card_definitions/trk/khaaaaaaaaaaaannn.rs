// Complete pinned Oracle and XMage class: the death replacement applies even
// when the first damage did not kill the creature.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DamageScaled(AmountValue::Multiple {
                value: &AmountValue::ChosenX,
                factor: 2,
            }),
            Instruction::ExileIfDiesThisTurn,
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
