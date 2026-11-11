pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfCastFromNonHand { skip: 2 },
            Instruction::CreateTokensThenCounters {
                token: Card(5467),
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: 1,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5467),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
