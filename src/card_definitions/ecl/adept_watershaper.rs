// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTappedKeywords {
            other: true,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    ..CardRules::EMPTY
};
