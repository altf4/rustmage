// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::LandMana {
            recipients: LandManaRecipients::YouControl,
            production: ManaProduction::any_color(1),
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Permanent,
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 2,
                },
                destination: MilledDestination::Hand,
            }),
        ],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5643), Card::from_catalogue_id(5644)],
            triggers: &[(0, RoomDoor::Right)],
            static_groups: &[(0, RoomDoor::Left)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
