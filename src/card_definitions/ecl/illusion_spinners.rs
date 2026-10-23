// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
const UNTAPPED_HEXPROOF: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::SourceUntapped),
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
    own_turn: false,
    keywords: Keywords::HEXPROOF,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, UNTAPPED_HEXPROOF)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    extras: &CardExtras {
        flash_if_control_subtype: Some("Faerie"),
        keyword_grants: &[UNTAPPED_HEXPROOF],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
