// Ordinary Drake token created by Drake Hatcher.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    deck_limit: None,
    ..CardRules::EMPTY
};
