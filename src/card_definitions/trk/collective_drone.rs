// Pinned Oracle and XMage: layer-7a power counts controlled artifact creatures in every zone.
const STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::CountAllKinds(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1 | 16,
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
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
