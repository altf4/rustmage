// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ManifestDreadRepeated(AmountValue::ChosenX),
            Instruction::PutCountersOnManifested {
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
