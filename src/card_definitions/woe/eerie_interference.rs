// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventCreatureDamageToControllerAndCreaturesThisTurn],
    ),
    ..CardRules::EMPTY
};
