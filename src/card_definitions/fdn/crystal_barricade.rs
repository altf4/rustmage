// Direct typed behavior from complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    keywords: Keywords::DEFENDER,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::PlayerHexproof],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::PreventOtherCreatureNoncombatDamage,
            )],
        },
    ],

    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::PlayerHexproof],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::Replacement(
                    0,
                    ReplacementSpec::PreventOtherCreatureNoncombatDamage,
                )],
            },
        ],
        replacement_effects: &[ReplacementSpec::PreventOtherCreatureNoncombatDamage],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
