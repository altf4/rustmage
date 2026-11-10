pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::OpponentControlsMoreLands,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::SelfEnters,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5238),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
