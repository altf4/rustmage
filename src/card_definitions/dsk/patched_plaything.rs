pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::DOUBLE_STRIKE,
    extras: &CardExtras {
        hand_cast_entry_counters: &[(CounterKind::MinusOne, 2)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
