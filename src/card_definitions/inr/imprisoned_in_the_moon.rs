// Direct Oracle/XMage port: one printed continuous effect spans its layers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(67))),
    keywords: Keywords::NONE,
    aura: Some(AuraSpec {
        kinds: 67,
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
                form: PermanentForm::Land,
                creature_types: &[],
                colors: &[],
                base: None,
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentMana(ManaProduction::Fixed([0, 0, 0, 0, 0, 1])),
        ],
    }],
    ..CardRules::EMPTY
};
