// Direct typed behavior from the full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StaticCastRestriction(0, 3)],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StaticCastRestriction(0, 3)],
        }],
        static_cast_restriction: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
