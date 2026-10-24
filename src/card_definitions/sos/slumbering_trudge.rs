// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    entry: EntryTapped::ChosenXAtMost(2),
    extras: &CardExtras {
        x_symbols: 1,
        x_complement_entry_counters: Some((CounterKind::Stun, 3)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
