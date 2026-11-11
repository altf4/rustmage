// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseNumberDestroyPowerAtLeast {
            minimum: 0,
            maximum: 10,
        }],
    ),
    ..CardRules::EMPTY
};
