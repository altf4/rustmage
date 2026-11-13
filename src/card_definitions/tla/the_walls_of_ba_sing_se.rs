// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 30),
    keywords: Keywords::DEFENDER,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: u16::MAX,
            other: true,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    ..CardRules::EMPTY
};
