const MERCENARY: Card = Card::from_catalogue_id(5612);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensScaled {
            token: MERCENARY,
            amount: AmountValue::ChosenX,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
