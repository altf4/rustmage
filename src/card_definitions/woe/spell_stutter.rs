pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CounterUnlessPayScaled {
            amount: AmountValue::Sum(&[
                AmountValue::Fixed(2),
                AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Faerie"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            ]),
            exile: false,
        }],
    ),
    ..CardRules::EMPTY
};
