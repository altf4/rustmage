// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceNonDemon),
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDeathCard(ReturnKind::Demon)],
    }],
    ..CardRules::EMPTY
};
