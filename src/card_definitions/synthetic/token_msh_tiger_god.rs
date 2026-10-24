// Complete pinned TigerGodToken recipe.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MAX_ONE_BLOCKER,
    deck_limit: None,
    ..CardRules::EMPTY
};
