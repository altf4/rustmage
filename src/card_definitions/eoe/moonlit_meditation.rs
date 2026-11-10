const COPY_FIRST: ReplacementSpec = ReplacementSpec::FirstTokensCopyAttached;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 17,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    aura: Some(AuraSpec {
        kinds: 17,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, COPY_FIRST)],
    }],
    extras: &CardExtras {
        replacement_effects: &[COPY_FIRST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
