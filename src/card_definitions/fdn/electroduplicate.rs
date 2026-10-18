pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
