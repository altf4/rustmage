// Direct typed behavior from the complete Oracle body and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::PartitionLibraryPiles(4)]),
    ..CardRules::EMPTY
};
