pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyardKinds(1),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5329),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
