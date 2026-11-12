// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::PartitionRevealedLibraryPiles(5),
            Instruction::CreateTokensThenCounters {
                token: Card::from_catalogue_id(5680),
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::MovedCount,
            },
        ],
    ),
    ..CardRules::EMPTY
};
