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
            first_layer: AbilityLayer::Control,
            fragments: &[StaticAbilityFragment::ControlAttached],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                    name: None,
                    form: PermanentForm::AddCreatureSubtypes,
                    creature_types: &["Assassin"],
                    colors: &[],
                    base: Some((1, 1)),
                }),
                StaticAbilityFragment::AttachmentKeywords(Keywords::DEATHTOUCH),
            ],
        },
    ],
    ..CardRules::EMPTY
};
