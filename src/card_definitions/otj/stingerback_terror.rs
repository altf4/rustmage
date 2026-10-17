// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
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
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING.union(Keywords::TRAMPLE),
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::ControllerHandSize,
                    stats: Stats {
                        power: -1,
                        toughness: -1,
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
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::FLYING.union(Keywords::TRAMPLE),
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::ControllerHandSize,
                        stats: Stats {
                            power: -1,
                            toughness: -1,
                        },
                    },
                )],
            },
        ],
        plot_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::ControllerHandSize,
            stats: Stats {
                power: -1,
                toughness: -1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
