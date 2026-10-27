// Oracle-correct recipe: the pinned Java helper omits the printed Soldier type.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    deck_limit: None,
    ..CardRules::EMPTY
};
