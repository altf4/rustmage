// Typed behavior preserved after full pinned Oracle/XMage and paid rules audit.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
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
    keywords: Keywords::DOUBLE_STRIKE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::DOUBLE_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Cda,
            fragments: &[StaticAbilityFragment::DynamicStats(
                0,
                DynamicStats {
                    power: Some(StatExpr {
                        value: StatValue::Colors(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
                        multiplier: 1,
                        offset: 0,
                    }),
                    toughness: None,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::DOUBLE_STRIKE,
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Cda,
                fragments: &[StaticAbilityFragment::DynamicStats(
                    0,
                    DynamicStats {
                        power: Some(StatExpr {
                            value: StatValue::Colors(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: false,
                            }),
                            multiplier: 1,
                            offset: 0,
                        }),
                        toughness: None,
                    },
                )],
            },
        ],
        dynamic_stats: Some(DynamicStats {
            power: Some(StatExpr {
                value: StatValue::Colors(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                multiplier: 1,
                offset: 0,
            }),
            toughness: None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
