pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::LoseLife(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacksAlone,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Grow {
                    subject: Subject::Entering,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Entering,
                    keywords: Keywords::DEATHTOUCH,
                },
            ],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card(5633), Card(5634)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
