// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::DestroyCreaturesExceptChosenType],
    ),
    ..CardRules::EMPTY
};
