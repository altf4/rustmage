pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(1566)),
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
