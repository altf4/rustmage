// Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6010),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
