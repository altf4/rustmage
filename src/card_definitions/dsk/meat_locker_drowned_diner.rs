pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 2,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 3),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5645), Card::from_catalogue_id(5646)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
