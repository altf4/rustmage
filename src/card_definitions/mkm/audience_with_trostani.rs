pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card(5682),
                amount: 1,
                tapped: false,
            },
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::DistinctControlledCreatureTokenNames),
            ),
        ],
    ),
    ..CardRules::EMPTY
};
