// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Destroy,
            Instruction::If {
                guard: StepGuard::GainedLife,
                skip: 1,
            },
            Instruction::LoseLifeTargetController(3),
        ],
    ),
    ..CardRules::EMPTY
};
