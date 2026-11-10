pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::VIGILANCE.union(Keywords::REACH),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ColoredLegendaryWebSlinging(
            ManaCost {
                colored: [1, 1, 0, 0, 1, 0],
                generic: 0,
            },
        )],
    }],
    ..CardRules::EMPTY
};
