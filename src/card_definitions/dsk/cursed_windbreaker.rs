const EQUIPPED: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 0,
        toughness: 0,
    },
    keywords: Keywords::FLYING,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ManifestDread,
            Instruction::AttachSourceToLastManifested,
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentKeywords(Keywords::FLYING)],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        equipment: Some(EQUIPPED),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
