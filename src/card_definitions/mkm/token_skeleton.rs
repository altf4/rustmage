pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    deck_limit: None,
    ..CardRules::EMPTY
};
