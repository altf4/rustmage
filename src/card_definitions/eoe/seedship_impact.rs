pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(16 | 32)),
        &[
            Instruction::IfTargetManaValueAtMost {
                maximum: 2,
                skip: 3,
            },
            Instruction::Destroy,
            Instruction::CreateTokens {
                token: Card(5238),
                amount: 1,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::Destroy,
        ],
    ),
    ..CardRules::EMPTY
};
