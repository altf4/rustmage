// The creating effect supplies the sampled X/X as the token's copiable base.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    deck_limit: None,
    ..CardRules::EMPTY
};
