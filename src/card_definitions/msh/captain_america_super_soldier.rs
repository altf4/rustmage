// Full Oracle/XMage: shield counters protect independently from printed abilities.
const HEROES: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::SourceCounters(CounterKind::Shield)),
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Hero"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::HEXPROOF,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FIRST_STRIKE,
        )],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, HEROES),
            StaticAbilityFragment::PlayerHexproofIfSourceCounter {
                kind: CounterKind::Shield,
                minimum: 1,
            },
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FIRST_STRIKE,
    static_groups: GROUPS,
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::Shield, 1)],
        keyword_grants: &[HEROES],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
