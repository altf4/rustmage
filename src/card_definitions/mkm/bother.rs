pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5266),
                amount: 3,
                tapped: false,
            },
            Instruction::Surveil(2),
        ],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(1654)),
        cost_alternatives: &[
            ManaCost {
                colored: [1, 1, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
