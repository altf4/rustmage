// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
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
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::LiveBase(
                    0,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Creatures {
                            relation: 0,
                            other: true,
                            enchanted_by_controller: true,
                        },
                        condition: None,
                        power: Some(LiveStatValue::Fixed(4)),
                        toughness: Some(LiveStatValue::Fixed(4)),
                        bonus_power: 0,
                        bonus_toughness: 0,
                        keywords: Keywords::FLYING,
                        cannot_block_subtype: None,
                    },
                ),
                StaticAbilityFragment::LiveKeywords(
                    0,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Creatures {
                            relation: 0,
                            other: true,
                            enchanted_by_controller: true,
                        },
                        condition: None,
                        power: Some(LiveStatValue::Fixed(4)),
                        toughness: Some(LiveStatValue::Fixed(4)),
                        bonus_power: 0,
                        bonus_toughness: 0,
                        keywords: Keywords::FLYING,
                        cannot_block_subtype: None,
                    },
                ),
            ],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[
                    StaticAbilityFragment::LiveBase(
                        0,
                        LiveStaticEffect {
                            recipients: LiveStatRecipients::Creatures {
                                relation: 0,
                                other: true,
                                enchanted_by_controller: true,
                            },
                            condition: None,
                            power: Some(LiveStatValue::Fixed(4)),
                            toughness: Some(LiveStatValue::Fixed(4)),
                            bonus_power: 0,
                            bonus_toughness: 0,
                            keywords: Keywords::FLYING,
                            cannot_block_subtype: None,
                        },
                    ),
                    StaticAbilityFragment::LiveKeywords(
                        0,
                        LiveStaticEffect {
                            recipients: LiveStatRecipients::Creatures {
                                relation: 0,
                                other: true,
                                enchanted_by_controller: true,
                            },
                            condition: None,
                            power: Some(LiveStatValue::Fixed(4)),
                            toughness: Some(LiveStatValue::Fixed(4)),
                            bonus_power: 0,
                            bonus_toughness: 0,
                            keywords: Keywords::FLYING,
                            cannot_block_subtype: None,
                        },
                    ),
                ],
            },
        ],
        live_static_effects: &[LiveStaticEffect {
            recipients: LiveStatRecipients::Creatures {
                relation: 0,
                other: true,
                enchanted_by_controller: true,
            },
            condition: None,
            power: Some(LiveStatValue::Fixed(4)),
            toughness: Some(LiveStatValue::Fixed(4)),
            bonus_power: 0,
            bonus_toughness: 0,
            keywords: Keywords::FLYING,
            cannot_block_subtype: None,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
