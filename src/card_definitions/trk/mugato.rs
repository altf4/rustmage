// Pinned Oracle and XMage: all three printed combat keywords are intrinsic.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::REACH
        .union(Keywords::VIGILANCE)
        .union(Keywords::DEATHTOUCH),
    ..CardRules::EMPTY
};
