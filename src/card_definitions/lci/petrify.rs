// The same Aura can restrict an artifact, a creature, or a permanent with both types.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Attached,
    attack: true,
    block: true,
    unless: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1 | 16,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    aura: Some(AuraSpec {
        kinds: 1 | 16,
        relation: -1,
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
        fragments: &[
            StaticAbilityFragment::CombatRestriction(0, RESTRICTION),
            StaticAbilityFragment::AttachedActivationsProhibited,
        ],
    }],
    extras: &CardExtras {
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
