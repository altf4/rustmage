pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    deck_limit: None,
    ..CardRules::EMPTY
};
