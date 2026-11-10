pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                object(16 | 32),
                TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Keyword(Keywords::FLYING),
                },
            ]),
            min: 0,
            max: 1,
        }),
        &[
            Instruction::Destroy,
            Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
