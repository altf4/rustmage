// Printed Star Trek token behavior; source pinned in the paired provenance record.
const BOOST: Stats = Stats {
    power: 1,
    toughness: 1,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::AttachmentStats(BOOST)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
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
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BOOST,
            keywords: Keywords::NONE,
        }),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    deck_limit: None,
    ..CardRules::EMPTY
};
