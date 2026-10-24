// Direct typed token recipe from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,

    deck_limit: None,
    ..CardRules::EMPTY
};
