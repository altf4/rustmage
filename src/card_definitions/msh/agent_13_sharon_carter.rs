// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
