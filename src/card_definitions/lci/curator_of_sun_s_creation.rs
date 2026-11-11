pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::DiscoveredYou),
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscoverScaled(
            AmountValue::DiscoverEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};
