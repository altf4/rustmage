pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
            Instruction::AdditionalLandPlays(1),
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2340)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
