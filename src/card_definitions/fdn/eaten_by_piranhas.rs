// Direct Oracle/XMage port: one printed continuous effect spans its layers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
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
                form: PermanentForm::Creature,
                creature_types: &["Skeleton"],
                colors: &[ChoiceColor::Black],
                base: Some((1, 1)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
        ],
    }],
    ..CardRules::EMPTY
};
