pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamagedAtLeast(3),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
