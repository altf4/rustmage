const EQUIP: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 3,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    sorcery: true,
    once_per_turn: false,
    targets: one(TargetRule::creature_you_control()),
    program: &[Instruction::AttachSource],
};

const BONUS: Stats = Stats {
    power: 1,
    toughness: 2,
};
const STATIC: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                ..EQUIP.cost
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::Subtype("Detective"),
            }),
            ..EQUIP
        },
        EQUIP,
    ],
    static_groups: STATIC,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        }),
        static_ability_groups: STATIC,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
