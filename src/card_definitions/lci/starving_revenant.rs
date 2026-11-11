const DESCEND: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1 | 2 | 16 | 32 | 64 | 256,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    minimum: Some(8),
    maximum: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SurveilRememberTop(2),
                Instruction::IfAmountAtLeast {
                    value: AmountValue::RememberedCount,
                    minimum: 1,
                    skip: 2,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 3),
                Instruction::IfAmountAtLeast {
                    value: AmountValue::RememberedCount,
                    minimum: 2,
                    skip: 2,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 3),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: DESCEND,
                event: &TriggerCondition::CardDrawn {
                    relation: CastRelation::You,
                    ordinal: None,
                },
            },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::LoseLife(PlayerSet::Targeted, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
