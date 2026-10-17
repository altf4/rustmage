// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
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
    keywords: Keywords(256),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(256))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::Sum(&[
                        StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 1,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
                        StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 1,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                    ]),
                    stats: Stats {
                        power: 1,
                        toughness: 1,
                    },
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(256))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::Sum(&[
                            StatValue::Count(CountFilter {
                                zone: CountZone::Battlefield,
                                relation: 0,
                                kind: 1,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: false,
                            }),
                            StatValue::Count(CountFilter {
                                zone: CountZone::Graveyard,
                                relation: 0,
                                kind: 1,
                                exclude: 0,
                                subtype: None,
                                basic: false,
                                other: false,
                                cards_only: true,
                            }),
                        ]),
                        stats: Stats {
                            power: 1,
                            toughness: 1,
                        },
                    },
                )],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::Sum(&[
                StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
            ]),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
