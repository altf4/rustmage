pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseOpposingCreatureForDeparture {
                other_power: -2,
                other_toughness: -2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ChosenCreatureLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
