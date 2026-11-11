const MAP: Card = Card::from_catalogue_id(5284);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::RememberTargetTapped,
            Instruction::ReturnToHand,
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: MAP,
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
