pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLYING,
    deck_limit: None,
    ..CardRules::EMPTY
};
