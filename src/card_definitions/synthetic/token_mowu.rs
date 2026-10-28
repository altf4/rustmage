// Named Mowu token from the pinned XMage source; admission awaits tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    deck_limit: None,
    ..CardRules::EMPTY
};
