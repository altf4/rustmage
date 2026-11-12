const EQUIPPED: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 3,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};
const CONTROLLED_CREATURE: TargetPlan = one(TargetRule::creature_you_control());

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: CONTROLLED_CREATURE,
        program: &[Instruction::AttachSource],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: CONTROLLED_CREATURE,
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(EQUIPPED.stats)],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        equipment: Some(EQUIPPED),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
