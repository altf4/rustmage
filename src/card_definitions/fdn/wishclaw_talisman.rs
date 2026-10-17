// Gated printed fixture for fixed named entry-counter machinery.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::Wish, 3)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
