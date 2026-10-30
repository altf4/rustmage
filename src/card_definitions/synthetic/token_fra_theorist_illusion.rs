// Pinned 1/1 blue Illusion token; gated until integration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    deck_limit: None,
    ..CardRules::EMPTY
};
