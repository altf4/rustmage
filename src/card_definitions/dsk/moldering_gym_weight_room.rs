// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ManifestDread,
                Instruction::PutCountersOnManifested {
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Fixed(3),
                },
            ],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5649), Card::from_catalogue_id(5650)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
