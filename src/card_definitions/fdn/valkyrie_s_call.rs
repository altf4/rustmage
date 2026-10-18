// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::ControlledNontokenNonAngel),
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDeathCard(ReturnKind::Angel)],
    }],
    ..CardRules::EMPTY
};
