pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[
            Instruction::Damage(3),
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(1),
        ],
    ),
    ..CardRules::EMPTY
};
