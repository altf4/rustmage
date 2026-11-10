// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE.union(Keywords::HASTE),
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    static_groups: &[
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledSpellsUncounterable(1)],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CombatDamageCantBePrevented],
        },
    ],
    ..CardRules::EMPTY
};
