// Exact shared named Doombot recipe from complete pinned Oracle and Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::NONE,
    deck_limit: None,
    ..CardRules::EMPTY
};
