pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::RoomWithoutSharedName,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(1),
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: StatValue::DifferentUnlockedRoomNames,
                        minimum: Some(8),
                        maximum: None,
                    },
                    skip: 1,
                },
                Instruction::WinGame,
            ],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5625), Card::from_catalogue_id(5626)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
