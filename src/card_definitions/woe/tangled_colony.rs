pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::CANT_BLOCK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card(5323),
            amount: AmountValue::SourceDamageReceivedThisTurn,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
