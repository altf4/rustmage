pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::If {
                guard: StepGuard::Outlaw,
                skip: 2,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -4,
                toughness: 0,
            },
            Instruction::Skip(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: -1,
                toughness: 0,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
