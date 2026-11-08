// Complete direct typed Oracle/XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 1,
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
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: None,
                form: PermanentForm::Preserve,
                creature_types: &["Turtle"],
                colors: &[],
                base: Some((0, 1)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::CombatRestriction(
                0,
                CombatRestriction {
                    subject: Subject::Attached,
                    attack: true,
                    block: false,
                    unless: None,
                },
            ),
        ],
    }],
    extras: &CardExtras {
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Attached,
            attack: true,
            block: false,
            unless: None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
