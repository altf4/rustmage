// Printed Star Trek token behavior; source pinned in the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::CANT_BLOCK,
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::CANT_BLOCK)] }],
    deck_limit: None,
    ..CardRules::EMPTY
};
