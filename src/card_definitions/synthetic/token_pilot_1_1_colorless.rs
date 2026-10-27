pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::CREW_SADDLE_PLUS_TWO,
    deck_limit: None,
    ..CardRules::EMPTY
};
