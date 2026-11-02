// Complete direct typed behavior from full pinned Oracle and XMage sources.
const MAX_SPEED: StatValue = StatValue::AtLeast {
    value: &StatValue::ControllerSpeed,
    minimum: 4,
};

const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING.union(Keywords::HASTE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: MAX_SPEED,
            keywords: Keywords::LIFELINK,
        }],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    static_groups: GROUPS,
    activated: &[],
    triggers: &[],

    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
