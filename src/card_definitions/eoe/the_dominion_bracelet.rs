const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachmentActivation {
                ability: 1,
                tap_attachment: false,
            },
        ],
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        attachment_activation_exile_costs: &[1],
        activation_generic_reductions: &[(1, ActivationGenericReduction::SourcePower)],
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 15,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::ControlTargetPlayerNextTurn],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
