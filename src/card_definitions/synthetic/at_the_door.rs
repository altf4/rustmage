pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensScaled {
            token: Card(5407),
            amount: AmountValue::ChosenX,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(151)),
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
