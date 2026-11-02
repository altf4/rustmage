// Direct typed behavior from complete pinned Oracle and XMage sources.
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 1,
        },
        keywords: Keywords::VIGILANCE,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(AURA),
    static_groups: &[
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 1,
                    toughness: 1,
                }),
                StaticAbilityFragment::AttachmentKeywords(Keywords::VIGILANCE),
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SelfGraveyardCastIf(
                StatValue::AtLeast {
                    value: &StatValue::ControllerSpeed,
                    minimum: 4,
                },
            )],
        },
    ],

    extras: &CardExtras {
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
