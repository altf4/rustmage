// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentAllKindsKeywords {
            kinds: 1 | 16,
            keywords: Keywords::DOUBLE_STRIKE,
        }],
    }],
    ..CardRules::EMPTY
};
