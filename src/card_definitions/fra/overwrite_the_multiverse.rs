// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileAllCreaturesCount,
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJaceScaled(AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};
