const OWN_TURN_HEXPROOF: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: true,
    keywords: Keywords::HEXPROOF,
};
const STATIC_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, OWN_TURN_HEXPROOF)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCombatDamageByGreaterToughness],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 6),
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        keyword_grants: &[OWN_TURN_HEXPROOF],
        static_ability_groups: STATIC_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
