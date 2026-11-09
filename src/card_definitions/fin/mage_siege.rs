pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5531),
            amount: 1,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
