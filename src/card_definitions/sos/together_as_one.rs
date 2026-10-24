// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec::one(TargetRule::Any),
        ]),
        &[
            Instruction::DrawScaled(PlayerSet::Targeted, AmountValue::ColorsSpent),
            Instruction::SelectGroup(1),
            Instruction::DamageScaled(AmountValue::ColorsSpent),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::ColorsSpent),
        ],
    ),
    ..CardRules::EMPTY
};
