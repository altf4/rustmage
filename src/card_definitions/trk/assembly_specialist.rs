// Pinned Oracle and XMage: one Borg artifact creature token on entry.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6007),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
