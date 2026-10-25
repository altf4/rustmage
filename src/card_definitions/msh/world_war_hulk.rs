// Partial real consumer for shared core tests; the whole card remains gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Chapter { number: 1 },
        targets: TargetPlan::NONE,
        program: &[Instruction::NextRedOrGreenCreatureFreeThisTurn],
    }],
    ..CardRules::EMPTY
};
