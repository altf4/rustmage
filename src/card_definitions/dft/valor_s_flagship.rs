// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
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
    keywords: Keywords(577),
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 7,
            toughness: 7,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
