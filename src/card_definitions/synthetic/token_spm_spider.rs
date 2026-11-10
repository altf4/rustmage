// Pinned synthetic token characteristics.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::REACH,
    deck_limit: None,
    ..CardRules::EMPTY
};
