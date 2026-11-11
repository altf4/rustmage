const POWER: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1 | 2 | 16 | 32 | 64 | 256,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(POWER),
    toughness: Some(StatExpr { offset: 1, ..POWER }),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    static_groups: GROUPS,
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        dynamic_stats: Some(STATS),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
