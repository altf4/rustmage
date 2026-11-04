// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::DoubleEnteredCreatureDamage,
            )],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::DoubleEnteredCreatureDamage],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
