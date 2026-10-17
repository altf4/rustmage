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
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::Counters {
                filter: ReplacementFilter::CreatureSpacecraftPlanet,
                kind: None,
                factor: 2,
                add: 0,
                effects_only: false,
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::Counters {
                    filter: ReplacementFilter::CreatureSpacecraftPlanet,
                    kind: None,
                    factor: 2,
                    add: 0,
                    effects_only: false,
                },
            )],
        }],
        warp_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        }),
        replacement_effects: &[ReplacementSpec::Counters {
            filter: ReplacementFilter::CreatureSpacecraftPlanet,
            kind: None,
            factor: 2,
            add: 0,
            effects_only: false,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
