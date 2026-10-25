// Real immutable core consumer; complete:false keeps this entire card gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CopyControlledNontokenCreaturesNonlegendary],
    ),
    ..CardRules::EMPTY
};
