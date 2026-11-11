const EQUIPMENT: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 0,
        toughness: 0,
    },
    keywords: Keywords::REACH,
};
const GRANTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
        StaticAbilityFragment::AttachmentActivation {
            ability: 1,
            tap_attachment: false,
        },
        StaticAbilityFragment::AttachmentActivation {
            ability: 2,
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
        ..CardExtras::EMPTY
    },
    activated: &[
        EQUIP,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1 | 16,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Tap],
            sorcery: false,
            ..EQUIP
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 8,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Discover(10)],
            sorcery: false,
            ..EQUIP
        },
    ],
    ..CardRules::EMPTY
};
