// Both independent life comparisons are captured when the attacker is declared.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacksPlayerWithMostLife,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5575),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::ControllerHasMostLife,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::SelfAttacks,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
