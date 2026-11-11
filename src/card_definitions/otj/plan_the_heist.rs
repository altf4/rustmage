// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::Hand,
                    minimum: None,
                    maximum: Some(0),
                },
                skip: 1,
            },
            Instruction::Surveil(3),
            Instruction::Draw(PlayerSet::Controller, 3),
        ],
    ),
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
