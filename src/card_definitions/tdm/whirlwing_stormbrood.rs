// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
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
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::CastTiming(0, CastFilter::Sorcery),
                StaticAbilityFragment::CastTiming(1, CastFilter::Subtype("Dragon")),
            ],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Rule,
                fragments: &[
                    StaticAbilityFragment::CastTiming(0, CastFilter::Sorcery),
                    StaticAbilityFragment::CastTiming(1, CastFilter::Subtype("Dragon")),
                ],
            },
        ],
        adventure_identity: Some("Whirlwing Stormbrood // Dynamic Soar"),
        omen_spell: Some(Card(5227)),
        cast_timing_grants: &[CastFilter::Sorcery, CastFilter::Subtype("Dragon")],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
