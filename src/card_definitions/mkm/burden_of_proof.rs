// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
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
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Base,
            fragments: &[
                StaticAbilityFragment::LiveBonus(
                    0,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Attached,
                        condition: Some(LiveStatCondition {
                            subtype: "Detective",
                            negated: false,
                        }),
                        power: None,
                        toughness: None,
                        bonus_power: 2,
                        bonus_toughness: 2,
                        keywords: Keywords::NONE,
                        cannot_block_subtype: None,
                    },
                ),
                StaticAbilityFragment::LiveBase(
                    1,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Attached,
                        condition: Some(LiveStatCondition {
                            subtype: "Detective",
                            negated: true,
                        }),
                        power: Some(LiveStatValue::Fixed(1)),
                        toughness: Some(LiveStatValue::Fixed(1)),
                        bonus_power: 0,
                        bonus_toughness: 0,
                        keywords: Keywords::NONE,
                        cannot_block_subtype: Some("Detective"),
                    },
                ),
                StaticAbilityFragment::LiveBlockRestriction(
                    1,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Attached,
                        condition: Some(LiveStatCondition {
                            subtype: "Detective",
                            negated: true,
                        }),
                        power: Some(LiveStatValue::Fixed(1)),
                        toughness: Some(LiveStatValue::Fixed(1)),
                        bonus_power: 0,
                        bonus_toughness: 0,
                        keywords: Keywords::NONE,
                        cannot_block_subtype: Some("Detective"),
                    },
                ),
            ],
        },
    ],
    extras: &CardExtras {
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
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Base,
                fragments: &[
                    StaticAbilityFragment::LiveBonus(
                        0,
                        LiveStaticEffect {
                            recipients: LiveStatRecipients::Attached,
                            condition: Some(LiveStatCondition {
                                subtype: "Detective",
                                negated: false,
                            }),
                            power: None,
                            toughness: None,
                            bonus_power: 2,
                            bonus_toughness: 2,
                            keywords: Keywords::NONE,
                            cannot_block_subtype: None,
                        },
                    ),
                    StaticAbilityFragment::LiveBase(
                        1,
                        LiveStaticEffect {
                            recipients: LiveStatRecipients::Attached,
                            condition: Some(LiveStatCondition {
                                subtype: "Detective",
                                negated: true,
                            }),
                            power: Some(LiveStatValue::Fixed(1)),
                            toughness: Some(LiveStatValue::Fixed(1)),
                            bonus_power: 0,
                            bonus_toughness: 0,
                            keywords: Keywords::NONE,
                            cannot_block_subtype: Some("Detective"),
                        },
                    ),
                    StaticAbilityFragment::LiveBlockRestriction(
                        1,
                        LiveStaticEffect {
                            recipients: LiveStatRecipients::Attached,
                            condition: Some(LiveStatCondition {
                                subtype: "Detective",
                                negated: true,
                            }),
                            power: Some(LiveStatValue::Fixed(1)),
                            toughness: Some(LiveStatValue::Fixed(1)),
                            bonus_power: 0,
                            bonus_toughness: 0,
                            keywords: Keywords::NONE,
                            cannot_block_subtype: Some("Detective"),
                        },
                    ),
                ],
            },
        ],
        live_static_effects: &[
            LiveStaticEffect {
                recipients: LiveStatRecipients::Attached,
                condition: Some(LiveStatCondition {
                    subtype: "Detective",
                    negated: false,
                }),
                power: None,
                toughness: None,
                bonus_power: 2,
                bonus_toughness: 2,
                keywords: Keywords::NONE,
                cannot_block_subtype: None,
            },
            LiveStaticEffect {
                recipients: LiveStatRecipients::Attached,
                condition: Some(LiveStatCondition {
                    subtype: "Detective",
                    negated: true,
                }),
                power: Some(LiveStatValue::Fixed(1)),
                toughness: Some(LiveStatValue::Fixed(1)),
                bonus_power: 0,
                bonus_toughness: 0,
                keywords: Keywords::NONE,
                cannot_block_subtype: Some("Detective"),
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
