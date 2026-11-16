// Pinned Oracle and XMage: live vigilance grant to controlled artifact creatures.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentAllKindsKeywords {
            kinds: 17,
            keywords: Keywords::VIGILANCE,
        }],
    }],
    ..CardRules::EMPTY
};
