pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
