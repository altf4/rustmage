// Exact token created by Magda, the Hoardmaster.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    ..CardRules::EMPTY
};
