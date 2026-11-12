const DEMONS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Demon"),
    basic: false,
    other: false,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: DEMONS,
                        minimum: Some(1),
                        maximum: None,
                    },
                    skip: 3,
                },
                Instruction::LoseLife(PlayerSet::Opponent, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
                Instruction::Skip(1),
                Instruction::LoseLife(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5671),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card(5665), Card(5666)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
