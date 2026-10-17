pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
