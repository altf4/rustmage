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
    keywords: Keywords(32768),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32768))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 3,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
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
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32768))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 3,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
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
            value: StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 3,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            }),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
