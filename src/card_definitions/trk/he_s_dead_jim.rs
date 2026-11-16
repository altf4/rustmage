// Pinned Oracle and XMage: the single target guards the entire spell on resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::Destroy, Instruction::Surveil(1)],
    ),
    ..CardRules::EMPTY
};
