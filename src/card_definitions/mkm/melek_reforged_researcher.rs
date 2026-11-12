const SIZE: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 4 | 8,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    multiplier: 2,
    offset: 0,
};

const BASE: DynamicStats = DynamicStats {
    power: Some(SIZE),
    toughness: Some(SIZE),
};

const DISCOUNT: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::InstantSorcery,
    amount: SpellReductionAmount::Fixed(3),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::InstantSorcerySpellsCastThisTurn,
        minimum: None,
        maximum: Some(0),
    },
};

const ABILITIES: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, BASE)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, DISCOUNT)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    static_groups: ABILITIES,
    extras: &CardExtras {
        dynamic_stats: Some(BASE),
        cost_reductions: &[DISCOUNT],
        static_ability_groups: ABILITIES,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
