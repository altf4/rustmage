pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Discover(10),
            Instruction::CreateTokensScaled {
                token: Card(5302),
                amount: AmountValue::DiscoverManaValueShortfall,
                tapped: true,
            },
        ],
    ),
    ..CardRules::EMPTY
};
