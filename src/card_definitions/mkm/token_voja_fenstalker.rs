pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::TRAMPLE,
    deck_limit: None,
    ..CardRules::EMPTY
};
