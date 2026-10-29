// Oracle-authored draft; gated because pinned XMage implementation is unavailable.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[
        Instruction::AllowJaceLoyaltyAnytime,
        Instruction::EnsureJaceToken,
        Instruction::EmpowerJace(8),
    ]),
    ..CardRules::EMPTY
};
