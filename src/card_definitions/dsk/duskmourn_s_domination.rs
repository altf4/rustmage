// Directly authored from the paired Oracle and pinned XMage sources.
const WEAKEN: Stats = Stats {
    power: -3,
    toughness: 0,
};
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: WEAKEN,
        keywords: Keywords::NONE,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Control,
        fragments: &[StaticAbilityFragment::ControlAttached],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentStats(WEAKEN),
        ],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AURA),
    static_groups: GROUPS,
    extras: &CardExtras {
        aura: Some(AURA),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
