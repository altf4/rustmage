pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensThenCounters {
            token: Card(5683),
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Sum(&[
                AmountValue::Fixed(2),
                AmountValue::Stat(StatValue::OwnedGraveyardOrFaceUpExileNameOrSubtype {
                    name: "Slime Against Humanity",
                    subtype: "Ooze",
                }),
            ]),
        }],
    ),
    deck_limit: None,
    ..CardRules::EMPTY
};
