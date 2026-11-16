// Oracle and pinned XMage reviewed. The death trigger survives its source leaving.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6007),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
