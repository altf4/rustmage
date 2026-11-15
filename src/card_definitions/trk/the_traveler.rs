// Oracle and pinned XMage reviewed. This is a characteristic-defining ability.
const STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::DistinctNames(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 2,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::REACH.union(Keywords::VIGILANCE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    keywords: Keywords::REACH.union(Keywords::VIGILANCE),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
