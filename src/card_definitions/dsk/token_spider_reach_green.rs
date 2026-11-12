pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH,
    deck_limit: None,
    ..CardRules::EMPTY
};
