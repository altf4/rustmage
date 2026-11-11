// A single union predicate counts each controlled artifact creature once.
const COUNT: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 16 | 1,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    }),
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(COUNT),
    toughness: Some(COUNT),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    deck_limit: None,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
