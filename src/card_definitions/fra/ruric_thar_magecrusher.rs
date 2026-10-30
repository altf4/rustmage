// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::REACH
        .union(Keywords::VIGILANCE)
        .union(Keywords::TRAMPLE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::REACH
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::TRAMPLE),
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::SourceHasNotDealtCombatDamage,
                keywords: Keywords::HEXPROOF,
            }],
        },
    ],
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
