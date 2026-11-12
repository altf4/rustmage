// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyCreaturesWithPowerAtLeast(3)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: u16::MAX,
            other: true,
            keywords: Keywords::HEXPROOF,
        }],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5653), Card::from_catalogue_id(5654)],
            triggers: &[(0, RoomDoor::Left)],
            static_groups: &[(0, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
