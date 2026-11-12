pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
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
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                    name: None,
                    form: PermanentForm::AddCardTypes(1 | 16),
                    creature_types: &["Toy"],
                    colors: &[],
                    base: Some((0, 2)),
                }),
                StaticAbilityFragment::RemoveAttachedAbilities,
            ],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachedFaceUpProhibited],
        },
    ],
    ..CardRules::EMPTY
};
