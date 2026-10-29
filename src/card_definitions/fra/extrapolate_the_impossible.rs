// Complete frozen Oracle draft; pinned XMage implementation is absent. Gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::RevealTwoOutsideCards]),
    ..CardRules::EMPTY
};
