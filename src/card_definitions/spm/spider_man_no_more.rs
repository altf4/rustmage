// Oracle and pinned XMage evidence are recorded in the paired provenance.
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
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: None,
                form: PermanentForm::Preserve,
                creature_types: &["Citizen"],
                colors: &[],
                base: Some((1, 1)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentKeywords(Keywords::DEFENDER),
        ],
    }],
    ..CardRules::EMPTY
};
