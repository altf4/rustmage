// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Any),
        &[
            Instruction::If {
                guard: StepGuard::VoidActive,
                skip: 2,
            },
            Instruction::Damage(3),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    ..CardRules::EMPTY
};
