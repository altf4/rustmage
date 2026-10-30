// Authored from frozen Oracle; pinned XMage has no card-specific class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::AllowJaceLoyaltyAnytime,
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(8),
        ],
    ),
    ..CardRules::EMPTY
};
