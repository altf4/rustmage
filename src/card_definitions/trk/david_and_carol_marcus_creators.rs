// Pinned Oracle and XMage: entry creates a tapped New Planet mana land.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6009),
            amount: 1,
            tapped: true,
        }],
    }],
    ..CardRules::EMPTY
};
