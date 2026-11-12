// Directly authored from the paired Oracle and pinned XMage sources.
const PREVENT_AND_MILL: ReplacementSpec = ReplacementSpec::PreventControlledOpponentDamageAndMill;
const MINDSKINNER_RULES: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::CANT_BE_BLOCKED,
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PREVENT_AND_MILL)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(10, 1),
    keywords: Keywords::CANT_BE_BLOCKED,
    static_groups: MINDSKINNER_RULES,
    extras: &CardExtras {
        static_ability_groups: MINDSKINNER_RULES,
        replacement_effects: &[PREVENT_AND_MILL],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
