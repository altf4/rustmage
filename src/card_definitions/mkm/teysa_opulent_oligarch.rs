pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfAmountAtLeast {
                    value: AmountValue::Stat(StatValue::OpponentLostLifeThisTurn),
                    minimum: 1,
                    skip: 1,
                },
                Instruction::CreateTokens {
                    token: Card::from_catalogue_id(5296),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(
                &TriggerCondition::ControlledPermanentWithSubtypeDies("Clue"),
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5477),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
