// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: 1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::TapAttached,
            Instruction::RemoveAllCountersFrom(Subject::Attached),
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Text,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: Some("Humble Merchant"),
                form: PermanentForm::Preserve,
                creature_types: &["Citizen"],
                colors: &[],
                base: Some((1, 1)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentMana(ManaProduction::Fixed([0, 0, 0, 0, 0, 1])),
        ],
    }],
    ..CardRules::EMPTY
};
