// Direct typed recipe from the pinned AngelVigilanceToken source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    deck_limit: None,
    ..CardRules::EMPTY
};
