// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 7),
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
    keywords: Keywords(2097152),
    extras: &CardExtras {
        adventure_identity: Some("Stormkeld Vanguard // Bear Down"),
        adventure_spell: Some(Card(5206)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
