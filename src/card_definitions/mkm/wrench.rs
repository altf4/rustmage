// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::VIGILANCE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Creature),
            program: &[Instruction::Tap],
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
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachmentKeywords(BONUS.keywords),
            StaticAbilityFragment::AttachmentActivation {
                ability: 0,
                tap_attachment: false,
            },
        ],
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
