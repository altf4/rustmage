// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLASH.union(Keywords::DOUBLE_STRIKE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledTappedKeywords {
                other: true,
                keywords: Keywords::HEXPROOF,
            }],
        },
    ],
    extras: &CardExtras {
        has_convoke: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
