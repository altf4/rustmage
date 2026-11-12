const GALLERY_BASE: LiveStaticEffect = LiveStaticEffect {
    recipients: LiveStatRecipients::Creatures {
        relation: 0,
        other: false,
        enchanted_by_controller: false,
    },
    condition: None,
    power: Some(LiveStatValue::ControllerCreatureCount),
    toughness: Some(LiveStatValue::ControllerCreatureCount),
    bonus_power: 0,
    bonus_toughness: 0,
    keywords: Keywords::NONE,
    cannot_block_subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackPlayerExcludingSubtype("Toy"),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5669), // Toy Token
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Base,
        fragments: &[StaticAbilityFragment::LiveBase(0, GALLERY_BASE)],
    }],
    extras: &CardExtras {
        live_static_effects: &[GALLERY_BASE],
        room: Some(&RoomSpec {
            halves: [Card::from_catalogue_id(5635), Card::from_catalogue_id(5636)],
            triggers: &[(0, RoomDoor::Left)],
            static_groups: &[(1, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
