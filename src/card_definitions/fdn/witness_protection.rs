// Direct Oracle/XMage port: one printed continuous effect spans its layers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    keywords: Keywords::NONE,
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
        first_layer: AbilityLayer::Text,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: Some("Legitimate Businessperson"),
                form: PermanentForm::Creature,
                creature_types: &["Citizen"],
                colors: &[ChoiceColor::Green, ChoiceColor::White],
                base: Some((1, 1)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
        ],
    }],
    ..CardRules::EMPTY
};
