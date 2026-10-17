// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
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
    keywords: Keywords(8),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(8))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::AtLeast {
                        value: &StatValue::SpellsCastThisTurn,
                        minimum: 2,
                    },
                    stats: Stats {
                        power: 2,
                        toughness: 0,
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
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(8))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::AtLeast {
                            value: &StatValue::SpellsCastThisTurn,
                            minimum: 2,
                        },
                        stats: Stats {
                            power: 2,
                            toughness: 0,
                        },
                    },
                )],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::AtLeast {
                value: &StatValue::SpellsCastThisTurn,
                minimum: 2,
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
