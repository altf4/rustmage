// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [1, 0, 0, 0, 0, 0],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5256),
                amount: 2,
                tapped: false,
            },
        ],
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
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
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
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS.stats)],
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
