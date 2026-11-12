pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddManaSplitScaled {
                first: ChoiceColor::Red,
                second: ChoiceColor::Green,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Raccoon"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Expend(4),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Expend(8),
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlay {
                amount: 2,
                next_turn: true,
            }],
        },
    ],
    ..CardRules::EMPTY
};
