const EQUIPPED: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Opponent, 1)],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(EQUIPPED.stats)],
    }],
    extras: &CardExtras {
        equipment: Some(EQUIPPED),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
