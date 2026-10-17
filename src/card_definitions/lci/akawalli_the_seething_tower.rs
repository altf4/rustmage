// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
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
    keywords: Keywords(0),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 371,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 4,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 2,
                        },
                    },
                ),
                StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: Some(StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 371,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 4,
                        }),
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: true,
                        own_turn: false,
                        keywords: Keywords(256),
                    },
                ),
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[
                StaticAbilityFragment::ScalingModifier(
                    1,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 371,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 8,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 2,
                        },
                    },
                ),
                StaticAbilityFragment::StaticRuleGrant(
                    1,
                    KeywordGrant {
                        condition: Some(StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 371,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 8,
                        }),
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: true,
                        own_turn: false,
                        keywords: Keywords(65536),
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
                fragments: &[
                    StaticAbilityFragment::ScalingModifier(
                        0,
                        ScalingModifier {
                            subject: Subject::Source,
                            value: StatValue::AtLeast {
                                value: &StatValue::Count(CountFilter {
                                    zone: CountZone::Graveyard,
                                    relation: 0,
                                    kind: 371,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 4,
                            },
                            stats: Stats {
                                power: 2,
                                toughness: 2,
                            },
                        },
                    ),
                    StaticAbilityFragment::KeywordGrant(
                        0,
                        KeywordGrant {
                            condition: Some(StatValue::AtLeast {
                                value: &StatValue::Count(CountFilter {
                                    zone: CountZone::Graveyard,
                                    relation: 0,
                                    kind: 371,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 4,
                            }),
                            requires_life_gain: false,
                            filter: CreatureFilter {
                                relation: 0,
                                other: false,
                                subtype: None,
                                keyword: Keywords(0),
                                attacking: false,
                                tokens: None,
                                artifact: false,
                            },
                            source_only: true,
                            own_turn: false,
                            keywords: Keywords(256),
                        },
                    ),
                ],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[
                    StaticAbilityFragment::ScalingModifier(
                        1,
                        ScalingModifier {
                            subject: Subject::Source,
                            value: StatValue::AtLeast {
                                value: &StatValue::Count(CountFilter {
                                    zone: CountZone::Graveyard,
                                    relation: 0,
                                    kind: 371,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 8,
                            },
                            stats: Stats {
                                power: 2,
                                toughness: 2,
                            },
                        },
                    ),
                    StaticAbilityFragment::StaticRuleGrant(
                        1,
                        KeywordGrant {
                            condition: Some(StatValue::AtLeast {
                                value: &StatValue::Count(CountFilter {
                                    zone: CountZone::Graveyard,
                                    relation: 0,
                                    kind: 371,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 8,
                            }),
                            requires_life_gain: false,
                            filter: CreatureFilter {
                                relation: 0,
                                other: false,
                                subtype: None,
                                keyword: Keywords(0),
                                attacking: false,
                                tokens: None,
                                artifact: false,
                            },
                            source_only: true,
                            own_turn: false,
                            keywords: Keywords(65536),
                        },
                    ),
                ],
            },
        ],
        scaling_modifiers: &[
            ScalingModifier {
                subject: Subject::Source,
                value: StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 371,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: 4,
                },
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
            },
            ScalingModifier {
                subject: Subject::Source,
                value: StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 371,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: 8,
                },
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
            },
        ],
        keyword_grants: &[
            KeywordGrant {
                condition: Some(StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 371,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: 4,
                }),
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                source_only: true,
                own_turn: false,
                keywords: Keywords(256),
            },
            KeywordGrant {
                condition: Some(StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 371,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: 8,
                }),
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                source_only: true,
                own_turn: false,
                keywords: Keywords(65536),
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
