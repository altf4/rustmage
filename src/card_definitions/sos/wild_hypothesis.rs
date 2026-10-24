// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokensWithCounters {
                token: Card::SosFractalToken,
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::ChosenX,
            },
            Instruction::Surveil(2),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
