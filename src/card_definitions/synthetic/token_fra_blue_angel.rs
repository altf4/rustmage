// Pinned BlueAngelToken identity; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)] }],
    deck_limit: None,
    ..CardRules::EMPTY
};
