// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnOwnedGraveyardNonlandPermanents],
    ),
    ..CardRules::EMPTY
};
