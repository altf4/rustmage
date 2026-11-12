// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureBonusPerSubtype {
            exclude: "Human",
            maximum: 10,
        }],
    }],
    ..CardRules::EMPTY
};
