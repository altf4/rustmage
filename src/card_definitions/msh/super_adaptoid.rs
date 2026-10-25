// Full frozen Oracle and pinned Java. Current legendary counts are shared across owned worlds.
const STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::LegendaryCreaturesYouControl,
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
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::GainMissingTargetKeywordCounters],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
