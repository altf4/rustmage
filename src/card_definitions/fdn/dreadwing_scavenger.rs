// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
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
            line: 2,
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
                                kind: 0,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 7,
                        },
                        stats: Stats {
                            power: 1,
                            toughness: 1,
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
                                kind: 0,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                            minimum: 7,
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
                        keywords: Keywords::DEATHTOUCH,
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
                line: 2,
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
                                    kind: 0,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 7,
                            },
                            stats: Stats {
                                power: 1,
                                toughness: 1,
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
                                    kind: 0,
                                    exclude: 0,
                                    subtype: None,
                                    basic: false,
                                    other: false,
                                    cards_only: true,
                                }),
                                minimum: 7,
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
                            keywords: Keywords::DEATHTOUCH,
                        },
                    ),
                ],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::AtLeast {
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
                minimum: 7,
            },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        keyword_grants: &[KeywordGrant {
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
                minimum: 7,
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
            keywords: Keywords::DEATHTOUCH,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
