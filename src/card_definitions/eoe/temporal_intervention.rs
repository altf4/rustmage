pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::RevealHandDiscard { excluded_kinds: 2 }],
    ),
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(2),
            turn: CostTurn::Any,
            guard: StepGuard::VoidActive,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
