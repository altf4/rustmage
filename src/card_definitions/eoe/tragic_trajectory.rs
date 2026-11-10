pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::If {
                guard: StepGuard::VoidActive,
                skip: 2,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -10,
                toughness: -10,
            },
            Instruction::Skip(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: -2,
                toughness: -2,
            },
        ],
    ),
    ..CardRules::EMPTY
};
