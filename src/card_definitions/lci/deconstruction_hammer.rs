const EQUIPMENT: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::NONE,
};
const GRANTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(EQUIPMENT.stats),
        StaticAbilityFragment::AttachmentActivation {
            ability: 1,
            tap_attachment: false,
        },
    ],
}];
const EQUIP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: one(TargetRule::creature_you_control()),
    program: &[Instruction::AttachSource],
    sorcery: true,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRANTS,
    extras: &CardExtras {
        equipment: Some(EQUIPMENT),
        static_ability_groups: GRANTS,
        attachment_activation_sacrifice_costs: &[1],
        ..CardExtras::EMPTY
    },
    activated: &[
        EQUIP,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Destroy],
            sorcery: false,
            ..EQUIP
        },
    ],
    ..CardRules::EMPTY
};
