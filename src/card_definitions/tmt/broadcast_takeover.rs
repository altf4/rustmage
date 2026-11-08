// Direct typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::TakeOpposingArtifactsUntilEndTurn],
    ),
    ..CardRules::EMPTY
};
