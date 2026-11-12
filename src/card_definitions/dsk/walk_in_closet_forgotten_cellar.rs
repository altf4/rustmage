// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayLandFromGraveyard],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantGraveyardSpellsAndExileCardsThisTurn],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5667), Card::from_catalogue_id(5668)],
            triggers: &[(0, RoomDoor::Right)],
            static_groups: &[(0, RoomDoor::Left)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
