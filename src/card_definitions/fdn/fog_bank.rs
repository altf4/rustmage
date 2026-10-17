pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    keywords: Keywords::FLYING.union(Keywords::DEFENDER).union(Keywords::PREVENT_COMBAT_TO).union(Keywords::PREVENT_COMBAT_BY),
    ..CardRules::EMPTY
};
