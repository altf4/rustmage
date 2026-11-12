pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::CardsLeaveGraveyard),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5670),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::ReturnToHand],
        },
    ],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card(5631), Card(5632)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
