const DISCOUNT: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::CommittedCrimeThisTurn,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, DISCOUNT)],
    }],
    extras: &CardExtras {
        cost_reductions: &[DISCOUNT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
