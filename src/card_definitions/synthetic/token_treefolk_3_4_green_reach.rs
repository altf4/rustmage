// Pinned TreefolkReachToken: ordinary 3/4 green Treefolk with reach.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::REACH,
    ..CardRules::EMPTY
};
