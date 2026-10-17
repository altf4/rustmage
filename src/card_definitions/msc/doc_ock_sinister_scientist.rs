// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
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
    keywords: Keywords::NONE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Base,
            fragments: &[StaticAbilityFragment::BaseStatSetter(
                0,
                BaseStatSetter {
                    power: Some(8),
                    toughness: Some(8),
                    own_turn: false,
                    condition: Some(StatValue::AtLeast {
                        value: &StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        minimum: 8,
                    }),
                },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: Some(StatValue::AtLeast {
                        value: &StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Villain"),
                            basic: false,
                            other: true,
                            cards_only: false,
                        }),
                        minimum: 1,
                    }),
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: true,
                    own_turn: false,
                    keywords: Keywords::HEXPROOF,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Base,
                fragments: &[StaticAbilityFragment::BaseStatSetter(
                    0,
                    BaseStatSetter {
                        power: Some(8),
                        toughness: Some(8),
                        own_turn: false,
                        condition: Some(StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 8,
                        }),
                    },
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: Some(StatValue::AtLeast {
                            value: &StatValue::Count(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 0,
                                exclude: 0,
                                subtype: Some("Villain"),
                                basic: false,
                                other: true,
                                cards_only: false,
                            }),
                            minimum: 1,
                        }),
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: true,
                        own_turn: false,
                        keywords: Keywords::HEXPROOF,
                    },
                )],
            },
        ],
        base_stat_setter: Some(BaseStatSetter {
            power: Some(8),
            toughness: Some(8),
            own_turn: false,
            condition: Some(StatValue::AtLeast {
                value: &StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: 8,
            }),
        }),
        keyword_grants: &[KeywordGrant {
            condition: Some(StatValue::AtLeast {
                value: &StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Villain"),
                    basic: false,
                    other: true,
                    cards_only: false,
                }),
                minimum: 1,
            }),
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: true,
            own_turn: false,
            keywords: Keywords::HEXPROOF,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
