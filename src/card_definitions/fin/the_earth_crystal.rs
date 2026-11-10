// Direct typed behavior from the paired Oracle and pinned XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Colors {
        filter: &CastFilter::Any,
        mask: 16,
    },
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const DOUBLE_COUNTERS: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Creature,
    kind: Some(CounterKind::PlusOne),
    factor: 2,
    add: 0,
    effects_only: false,
};
const STATIC: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE_COUNTERS)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: STATIC,
    extras: &CardExtras {
        static_ability_groups: STATIC,
        cost_reductions: &[REDUCTION],
        replacement_effects: &[DOUBLE_COUNTERS],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 4,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 1,
            max: 2,
        }),
        program: &[Instruction::DistributeCounters {
            total: 2,
            kind: CounterKind::PlusOne,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
