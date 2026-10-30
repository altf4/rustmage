// Arc of Fortune prepared face; gated pending validation.
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
        prepared_identity: Some("@prepare/variable_chaser_arc_of_fortune"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
