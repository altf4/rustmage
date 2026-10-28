// Typed Oracle behavior; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5437)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
