const ISLAND_COUNT: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: Some("Island"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(ISLAND_COUNT),
    toughness: Some(ISLAND_COUNT),
};
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::InstantSorcery,
    amount: SpellReductionAmount::ColoredStat {
        color: ChoiceColor::Blue,
        value: StatValue::ControlledLandsWithCounter(CounterKind::Flood),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::InstantSorcerySpellsCastThisTurn,
        minimum: None,
        maximum: Some(0),
    },
};
const FLOOD: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SelfEnters,
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 2,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    program: &[
        Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Flood,
            amount: 1,
        },
        Instruction::AddBasicLandTypeWhileCounter {
            subject: Subject::Targets,
            kind: BasicLandType::Island,
            counter: CounterKind::Flood,
        },
    ],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[
        FLOOD,
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            ..FLOOD
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Cda,
            fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
        },
    ],
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
