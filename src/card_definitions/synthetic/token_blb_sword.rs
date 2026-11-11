const BOOST: Stats = Stats {
    power: 1,
    toughness: 1,
};
const EQUIPPED: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::AttachmentStats(BOOST)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    static_groups: EQUIPPED,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BOOST,
            keywords: Keywords::NONE,
        }),
        static_ability_groups: EQUIPPED,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
