pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    deck_limit: None,
    ..CardRules::EMPTY
};
