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
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(
            0,
            StaticModifier {
                own_turn: false,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords(0),
                attacking: true,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::StaticModifier(
                0,
                StaticModifier {
                    own_turn: false,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords(0),
                        attacking: true,
                        tokens: None,
                        artifact: false,
                    },
                    stats: Stats {
                        power: 1,
                        toughness: 0,
                    },
                },
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
