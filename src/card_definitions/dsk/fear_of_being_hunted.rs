pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::HASTE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MustBeBlocked],
    }],
    ..CardRules::EMPTY
};
