// Direct typed back face from the paired Oracle and XMage sources.
const STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::CraftMaterialPower,
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
