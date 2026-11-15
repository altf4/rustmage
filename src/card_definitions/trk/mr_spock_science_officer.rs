// Complete pinned Oracle and XMage MrSpockScienceOfficer class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::DoubleDraw],
    }],
    ..CardRules::EMPTY
};
