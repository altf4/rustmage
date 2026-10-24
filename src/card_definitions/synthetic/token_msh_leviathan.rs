// The complete pinned 6/5 blue Leviathan token recipe.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(6, 5),
    keywords: Keywords::HEXPROOF,
    ..CardRules::EMPTY
};
