// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeavesBattlefield,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
