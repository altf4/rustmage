// Oracle and official release notes supply this definition; XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::RevealTwoOutsideCards]),
    ..CardRules::EMPTY
};
