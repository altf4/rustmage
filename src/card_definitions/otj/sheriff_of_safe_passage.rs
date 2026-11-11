pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    extras: &CardExtras {
        entry_counter_rules: &[
            EntryCounterRule {
                guard: StepGuard::Always,
                amount: EntryCounterAmount::Fixed(1),
            },
            EntryCounterRule {
                guard: StepGuard::Always,
                amount: EntryCounterAmount::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: true,
                    cards_only: false,
                })),
            },
        ],
        plot_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
