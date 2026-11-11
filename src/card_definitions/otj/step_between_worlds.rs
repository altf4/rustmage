pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::BeginOptionalShuffleWheel(7),
            Instruction::AskOptionalWheel {
                first: true,
                skip: 2,
            },
            Instruction::RecordOptionalWheel {
                first: true,
                accept: true,
            },
            Instruction::Skip(1),
            Instruction::RecordOptionalWheel {
                first: true,
                accept: false,
            },
            Instruction::AskOptionalWheel {
                first: false,
                skip: 2,
            },
            Instruction::RecordOptionalWheel {
                first: false,
                accept: true,
            },
            Instruction::Skip(1),
            Instruction::RecordOptionalWheel {
                first: false,
                accept: false,
            },
            Instruction::ExecuteOptionalWheel,
            Instruction::ExileResolvingSpell,
        ],
    ),
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
