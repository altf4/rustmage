// Direct typed behavior from the paired Oracle and pinned XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Colors {
        filter: &CastFilter::Any,
        mask: 2,
    },
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
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
        fragments: &[StaticAbilityFragment::OpponentMillAdditional(4)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: STATIC,
    extras: &CardExtras {
        static_ability_groups: STATIC,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 4,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Mill(
            PlayerSet::Opponent,
            MillAmount::Amount(AmountValue::Stat(StatValue::ControllerHandSize)),
        )],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
