pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
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
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentTurnStats(Stats {
                power: 1,
                toughness: 0,
            }),
            StaticAbilityFragment::AttachedTurnKeywords(Keywords::FIRST_STRIKE),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        attached_turn_keywords: Keywords::FIRST_STRIKE,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
