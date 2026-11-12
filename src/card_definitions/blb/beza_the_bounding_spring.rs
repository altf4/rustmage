const LIFE_TIE_OR_AHEAD: StatValue = StatValue::ControllerHasMostLife;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::OpponentControlsMoreLands,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Not(&LIFE_TIE_OR_AHEAD),
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
            Instruction::If {
                guard: StepGuard::OpponentControlsMoreCreatures,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(5317),
                amount: 2,
                tapped: false,
            },
            Instruction::If {
                guard: StepGuard::OpponentHasMoreCardsInHand,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
