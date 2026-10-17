// Direct typed behavior from complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords(4097),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(4097))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledSpellsUncounterable(8 | 4)],
        },
    ],

    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(4097))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::ControlledSpellsUncounterable(8 | 4)],
            },
        ],
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
