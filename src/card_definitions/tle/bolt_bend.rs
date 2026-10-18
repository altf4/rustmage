pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::SingleTargetStackObject)),
        &[Instruction::ChangeSingleStackTarget],
    ),
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(3),
            turn: CostTurn::Any,
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
