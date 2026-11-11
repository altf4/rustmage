pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensScaled {
            token: Card(5284),
            amount: AmountValue::Sum(&[
                AmountValue::Fixed(1),
                AmountValue::Stat(StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 1,
                        kind: 16,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: 1,
                }),
            ]),
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2339)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
