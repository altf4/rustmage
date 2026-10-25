// Exact ordinary blue-only Merfolk token; old white-blue factory is unchanged.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::NONE,
    deck_limit: None,
    ..CardRules::EMPTY
};
