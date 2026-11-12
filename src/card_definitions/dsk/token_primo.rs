pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    deck_limit: None,
    ..CardRules::EMPTY
};
