// A 3/2 red and white Spirit creature token.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    deck_limit: None,
    ..CardRules::EMPTY
};
