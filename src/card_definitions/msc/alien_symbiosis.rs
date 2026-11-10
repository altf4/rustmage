const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::MENACE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: BONUS,
    }),
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedSubtype("Symbiote"),
                StaticAbilityFragment::AttachmentStats(BONUS.stats),
                StaticAbilityFragment::AttachmentKeywords(BONUS.keywords),
            ],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::SelfGraveyardCastWithAdditionalCosts {
                    life: 0,
                    additional: AdditionalCost {
                        kind: EffectChoiceKind::Discard,
                        filter: SacrificeFilter {
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 0,
                            nonland: false,
                            other: false,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        },
                    },
                },
            ],
        },
    ],
    extras: &CardExtras {
        attached_subtype: Some("Symbiote"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
