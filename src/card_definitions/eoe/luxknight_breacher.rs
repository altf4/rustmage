// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 17,
                exclude: 0,
                subtype: None,
                basic: false,
                other: true,
                cards_only: false,
            })),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
