const FUNGI: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: Some("Fungus"),
    basic: false,
    other: false,
    cards_only: false,
};
const SAPROLINGS: CountFilter = CountFilter {
    subtype: Some("Saproling"),
    ..FUNGI
};
const SIZE: StatExpr = StatExpr {
    value: StatValue::CountUnion(&[FUNGI, SAPROLINGS]),
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(SIZE),
    toughness: Some(SIZE),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::TRAMPLE,
    static_groups: GROUPS,
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card::from_catalogue_id(5292),
            amount: AmountValue::Stat(StatValue::DescendedThisTurn),
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
