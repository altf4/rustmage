// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::CreateTokensWithCounters {
                token: Card(5532),
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::Fixed(2),
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5532),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
