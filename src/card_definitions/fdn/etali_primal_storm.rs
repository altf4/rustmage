// Direct typed Oracle behavior; full pinned XMage source and dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileEachLibraryTopCastFree],
    }],
    ..CardRules::EMPTY
};
