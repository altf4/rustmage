const POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
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
    }),
    toughness: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(1, POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
