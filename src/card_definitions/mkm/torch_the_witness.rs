pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DamageForExcess(AmountValue::Sum(&[
                AmountValue::ChosenX,
                AmountValue::ChosenX,
            ])),
            Instruction::IfNoExcessDamage { skip: 1 },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
