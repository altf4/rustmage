const LANDS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }),
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Expression {
                value: StatValue::UntappedCount(LANDS),
                minimum: Some(5),
                maximum: None,
            },
            amount: EntryCounterAmount::FixedCounters(&[
                (CounterKind::PlusOne, 2),
                (CounterKind::Lifelink, 1),
            ]),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
