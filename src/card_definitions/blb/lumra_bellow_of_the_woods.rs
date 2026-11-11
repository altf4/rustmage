// Direct typed behavior from complete pinned Oracle and XMage sources.
const LANDS: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
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
};
const STATS: DynamicStats = DynamicStats {
    power: Some(LANDS),
    toughness: Some(LANDS),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::REACH.union(Keywords::VIGILANCE),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::ReturnOwnedGraveyardLandsTapped,
        ],
    }],
    ..CardRules::EMPTY
};
