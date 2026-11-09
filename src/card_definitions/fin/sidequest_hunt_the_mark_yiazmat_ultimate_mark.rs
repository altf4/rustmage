// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Destroy],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::OpponentControlledCreatureDied,
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::Stat(StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Treasure"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    })),
                    minimum: 3,
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
