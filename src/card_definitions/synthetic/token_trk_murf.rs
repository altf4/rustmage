// Printed Star Trek token behavior; source pinned in the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    keywords: Keywords::INDESTRUCTIBLE,
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::INDESTRUCTIBLE)] }],
    deck_limit: None,
    ..CardRules::EMPTY
};
