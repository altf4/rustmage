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
    keywords: Keywords(65),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(65))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::DoubleTokens {
                    effects_only: false,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(65))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::Replacement(
                    0,
                    ReplacementSpec::DoubleTokens {
                        effects_only: false,
                    },
                )],
            },
        ],
        warp_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }),
        replacement_effects: &[ReplacementSpec::DoubleTokens {
            effects_only: false,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
