// Direct typed behavior from the pinned Oracle and XMage sources.
const GRAVEYARD_CREATURES: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: -1,
        kind: 1,
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
    power: Some(GRAVEYARD_CREATURES),
    toughness: Some(GRAVEYARD_CREATURES),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5549)),
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
