// Directly authored from the paired Oracle and pinned XMage sources.
const DOUBLE_OVERDRIVE: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 0,
    },
    keywords: Keywords::DOUBLE_STRIKE,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(DOUBLE_OVERDRIVE.stats),
        StaticAbilityFragment::AttachmentKeywords(DOUBLE_OVERDRIVE.keywords),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        equipment: Some(DOUBLE_OVERDRIVE),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
