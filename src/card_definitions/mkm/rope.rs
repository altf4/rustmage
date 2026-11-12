const EQUIPPED_BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 2,
    },
    keywords: Keywords::REACH,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(EQUIPPED_BONUS.stats),
            StaticAbilityFragment::AttachmentKeywords(EQUIPPED_BONUS.keywords),
            StaticAbilityFragment::AttachmentRule(Keywords::MAX_ONE_BLOCKER),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EQUIPPED_BONUS),
        ..CardExtras::EMPTY
    },
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
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
