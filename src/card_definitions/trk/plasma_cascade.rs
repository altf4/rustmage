// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Any),
        &[Instruction::Damage(3), Instruction::DiscardUpToOneDraw],
    ),
    ..CardRules::EMPTY
};
