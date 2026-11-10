pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::If {
                guard: StepGuard::VoidActive,
                skip: 2,
            },
            Instruction::LoseLife(PlayerSet::Opponent, 2),
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
