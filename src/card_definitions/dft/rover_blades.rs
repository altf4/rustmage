// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 4,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::DOUBLE_STRIKE,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::DOUBLE_STRIKE,
        }),
        vehicle_stats: Some(Stats {
            power: 2,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
