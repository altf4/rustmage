// Complete Falcon and its exact Redwing factory use ordinary typed programs.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5393),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
