// Printed Star Trek token behavior; source pinned in the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    deck_limit: None,
    ..CardRules::EMPTY
};
