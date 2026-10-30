// Typed Oracle behavior; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::NONE,
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        }],
        enters_prepared: true,
        prepared_spell: Some(Card(5442)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
