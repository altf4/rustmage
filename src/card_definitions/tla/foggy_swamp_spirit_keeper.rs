pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5705),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
