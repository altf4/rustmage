// Reserved immutable TDM token identity; authored behavior and proof pending.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    deck_limit: None,
    ..CardRules::EMPTY
};
