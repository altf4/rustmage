// Pinned FRA Leviathan token; gated pending integration and rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    keywords: Keywords::HEXPROOF,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HEXPROOF)],
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
