pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayerProtectionFromOpponents],
    }],
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
