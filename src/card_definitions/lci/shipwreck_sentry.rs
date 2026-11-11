const STATIC_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackWithoutDefenderIf(
            StatValue::ArtifactEnteredThisTurn,
        )],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEFENDER,
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        static_ability_groups: STATIC_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
