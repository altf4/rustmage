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
    keywords: Keywords(0),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::DoubleTokens { effects_only: true },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                1,
                ReplacementSpec::Counters {
                    filter: ReplacementFilter::Permanent,
                    kind: None,
                    factor: 2,
                    add: 0,
                    effects_only: true,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::Replacement(
                    0,
                    ReplacementSpec::DoubleTokens { effects_only: true },
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::Replacement(
                    1,
                    ReplacementSpec::Counters {
                        filter: ReplacementFilter::Permanent,
                        kind: None,
                        factor: 2,
                        add: 0,
                        effects_only: true,
                    },
                )],
            },
        ],
        replacement_effects: &[
            ReplacementSpec::DoubleTokens { effects_only: true },
            ReplacementSpec::Counters {
                filter: ReplacementFilter::Permanent,
                kind: None,
                factor: 2,
                add: 0,
                effects_only: true,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
