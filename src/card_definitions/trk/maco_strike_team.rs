pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentTargetsControlledCreature,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(6008),
            amount: 1,
            tapped: true,
        }],
    }],
    ..CardRules::EMPTY
};
