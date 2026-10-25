// Directly authored full frozen Oracle and pinned XMage body; shared rules are typed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CopyControlledNontokenCreaturesNonlegendary],
    ),
    ..CardRules::EMPTY
};
