// Whole-card Oracle behavior; source and casualty keyword implementation pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING.union(Keywords::VIGILANCE),
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledSpellsCasualty {
                kinds: 12,
                minimum: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
