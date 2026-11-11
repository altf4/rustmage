const TARGET: TargetRule = TargetRule::Union(&[
    TargetRule::Creature,
    TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Subtype("Food"),
    },
]);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TARGET))),
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
    extras: &CardExtras {
        aura_kind_or_subtype: Some("Food"),
        ..CardExtras::EMPTY
    },
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                    name: None,
                    form: PermanentForm::FoodArtifact,
                    creature_types: &[],
                    colors: &[],
                    base: None,
                }),
                StaticAbilityFragment::RemoveAttachedAbilities,
                StaticAbilityFragment::AttachmentActivation {
                    ability: 0,
                    tap_attachment: false,
                },
            ],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
