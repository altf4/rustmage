const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Threshold {
        value: StepValue::Life,
        minimum: None,
        maximum: Some(5),
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(1 | 64)),
        &[Instruction::ExileTargetAndSearchItsController],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
