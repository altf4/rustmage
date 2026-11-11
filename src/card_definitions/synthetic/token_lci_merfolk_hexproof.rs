// Direct typed recipe from the pinned MerfolkHexproofToken source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::HEXPROOF,
    deck_limit: None,
    ..CardRules::EMPTY
};
