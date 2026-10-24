// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokensWithCounters {
                token: Card::SosFractalToken,
                amount: 2,
                kind: CounterKind::PlusOne,
                counters: AmountValue::ColorsSpent,
            },
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::ColorsSpent),
        ],
    ),
    ..CardRules::EMPTY
};
