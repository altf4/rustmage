pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ManifestDread,
                Instruction::PutCountersOnManifested {
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Fixed(1),
                },
            ],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5663), Card::from_catalogue_id(5664)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
