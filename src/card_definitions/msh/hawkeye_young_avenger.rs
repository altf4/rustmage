// Full frozen Oracle and pinned Java; signed power follows CR107.1b.
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::NoncombatDamagePlusSourcePower,
        )],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::REACH,
    static_groups: GROUPS,
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::NoncombatDamagePlusSourcePower],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
