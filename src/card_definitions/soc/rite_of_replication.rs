pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::IfKicked { skip: 2 },
            Instruction::CreateCopyToken {
                subject: Subject::Targets,
                amount: 5,
            },
            Instruction::Skip(1),
            Instruction::CreateCopyToken {
                subject: Subject::Targets,
                amount: 1,
            },
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0; 6],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
