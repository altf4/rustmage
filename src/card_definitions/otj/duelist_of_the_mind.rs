const STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::CardsDrawnThisTurn,
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayDraw { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
