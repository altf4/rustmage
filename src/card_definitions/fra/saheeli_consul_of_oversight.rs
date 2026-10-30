// Directly authored from pinned Oracle and XMage; pending focused validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::ScryOrSurveil),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5266),
            amount: 1,
            tapped: false,
        }],
    }],
    keywords: Keywords::FLYING,
    ..CardRules::EMPTY
};
