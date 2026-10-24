// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensWithCounters {
            token: Card::SosFractalToken,
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Stat(StatValue::CardsDrawnThisTurn),
        }],
    ),
    ..CardRules::EMPTY
};
