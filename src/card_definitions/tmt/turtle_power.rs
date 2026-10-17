// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
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
    keywords: Keywords(32),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::StaticModifier(
                0,
                StaticModifier {
                    own_turn: false,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: Some("Turtle"),
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    stats: Stats {
                        power: 2,
                        toughness: 2,
                    },
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some("Turtle"),
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 2,
                toughness: 2,
            },
        }],
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::StaticModifier(
                    0,
                    StaticModifier {
                        own_turn: false,
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: Some("Turtle"),
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 2,
                        },
                    },
                )],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
