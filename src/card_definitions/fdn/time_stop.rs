// Complete pinned Oracle/XMage body; shared cleanup and ordered SBA machinery.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::EndTurn],
    ),
    ..CardRules::EMPTY
};
