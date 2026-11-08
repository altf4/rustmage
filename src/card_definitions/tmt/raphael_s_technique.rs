// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::BeginOptionalWheel(7),
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
        ],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
