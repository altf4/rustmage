// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::DamageScaled(AmountValue::Stat(
                StatValue::ControllerHandSize,
            ))],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NoMaximumHandSize],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5655), Card::from_catalogue_id(5656)],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[(0, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
