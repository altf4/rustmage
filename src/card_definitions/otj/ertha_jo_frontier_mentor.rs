const MERCENARY: Card = Card::from_catalogue_id(5612);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: MERCENARY,
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AbilityActivatedTargetingCreatureOrPlayer,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedActivatedAbility],
        },
    ],
    ..CardRules::EMPTY
};
