// Complete printed behavior. Base characteristics come from the pinned facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::DOUBLE_STRIKE.union(Keywords::CANT_BE_BLOCKED),
    ..CardRules::EMPTY
};
