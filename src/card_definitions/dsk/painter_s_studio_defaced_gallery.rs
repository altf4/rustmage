pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlay {
                amount: 2,
                next_turn: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                power: 1,
                toughness: 0,
            }],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card(5651), Card(5652)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
