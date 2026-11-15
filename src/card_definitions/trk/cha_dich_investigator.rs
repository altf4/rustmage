// Pinned Oracle and XMage: one haste Klingon Warrior when this creature enters.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6010),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
