// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
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
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(
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
                    toughness: 0,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
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
                        toughness: 0,
                    },
                },
            )],
        }],
        scaling_modifiers: &[ScalingModifier {
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
                toughness: 0,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
