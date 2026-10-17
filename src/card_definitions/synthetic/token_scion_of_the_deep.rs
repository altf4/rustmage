pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    deck_limit: None,
    ..CardRules::EMPTY
};
