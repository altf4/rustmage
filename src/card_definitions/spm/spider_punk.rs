// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::RIOT,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::GrantRiotToOtherControlledSubtype(
                "Spider",
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StackItemsCannotBeCountered],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::DamageCantBePrevented],
        },
    ],
    ..CardRules::EMPTY
};
