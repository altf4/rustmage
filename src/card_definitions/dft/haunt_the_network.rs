// Create the artifacts before measuring the drain, including those newly created tokens.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::CreateTokens {
                token: Card(5266),
                amount: 2,
                tapped: false,
            },
            Instruction::LoseOpponentGainSame(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }))),
        ],
    ),
    ..CardRules::EMPTY
};
