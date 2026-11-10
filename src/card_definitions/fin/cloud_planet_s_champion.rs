const OWN_TURN_EQUIPPED: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::SourceEquipmentCount),
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
    source_only: true,
    own_turn: true,
    keywords: Keywords::DOUBLE_STRIKE.union(Keywords::INDESTRUCTIBLE),
};
const STATIC: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, OWN_TURN_EQUIPPED)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::EquipTargetReduction(2)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: STATIC,
    extras: &CardExtras {
        static_ability_groups: STATIC,
        keyword_grants: &[OWN_TURN_EQUIPPED],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
