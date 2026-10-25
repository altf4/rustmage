// Partial real consumer for shared core tests; the whole card remains gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Chapter { number: 2 },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReduceArtifactSpellCostsThisTurn(1)],
    }],
    ..CardRules::EMPTY
};
