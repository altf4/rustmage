pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: crate::engine::CopyExceptions {
                reflection: true,
                ..crate::engine::CopyExceptions::NONE
            },
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanent,
            condition: StatValue::Fixed(1),
        }],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5647), Card::from_catalogue_id(5648)],
            triggers: &[(0, RoomDoor::Left)],
            static_groups: &[(1, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
