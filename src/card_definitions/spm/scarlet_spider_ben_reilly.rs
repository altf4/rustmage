// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::TRAMPLE,
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 1, 0],
            generic: 0,
        }),
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::WebSlingingReturnedManaValue,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
