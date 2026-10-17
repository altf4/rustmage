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
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
