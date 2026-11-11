// Direct typed recipe from the pinned VampireDemonToken source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING,
    deck_limit: None,
    ..CardRules::EMPTY
};
