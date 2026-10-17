// Flash Aura: the -5/-0 modifier and loss share one continuous effect.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec { stats: Stats { power: -5, toughness: 0 }, keywords: Keywords::NONE },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentStats(Stats { power: -5, toughness: 0 }),
        ],
    }],
    ..CardRules::EMPTY
};
