// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::FLYING
        .union(Keywords::VIGILANCE)
        .union(Keywords::TRAMPLE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::TRAMPLE),
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::SourceHasNotDealtDamage,
                keywords: Keywords::HEXPROOF,
            }],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::FLYING
                        .union(Keywords::VIGILANCE)
                        .union(Keywords::TRAMPLE),
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                    condition: StatValue::SourceHasNotDealtDamage,
                    keywords: Keywords::HEXPROOF,
                }],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
