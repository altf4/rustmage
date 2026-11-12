// Directly authored from the paired Oracle and pinned XMage sources.
const BONUS: Stats = Stats {
    power: 1,
    toughness: 1,
};
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: 0,
    bonus: EquipmentSpec {
        stats: BONUS,
        keywords: Keywords::NONE,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentEntryTurnKeywords(
            Keywords::HEXPROOF,
        )],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    keywords: Keywords::FLASH,
    aura: Some(AURA),
    static_groups: GROUPS,
    extras: &CardExtras {
        aura: Some(AURA),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
