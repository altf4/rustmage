// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::ManifestDread],
    }],
    ..CardRules::EMPTY
};
