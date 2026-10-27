// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        }),
        as_turned_face_up_counters: &[(CounterKind::PlusOne, 4)],
        ..CardExtras::EMPTY
    },
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
    ..CardRules::EMPTY
};
