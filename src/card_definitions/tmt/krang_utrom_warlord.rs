// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(9, 9),
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
    keywords: Keywords::FLYING
        .union(Keywords::HASTE)
        .union(Keywords::TRAMPLE)
        .union(Keywords::INDESTRUCTIBLE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING
                    .union(Keywords::HASTE)
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::INDESTRUCTIBLE),
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: None,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: true,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: true,
                    },
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords::FLYING
                        .union(Keywords::HASTE)
                        .union(Keywords::TRAMPLE)
                        .union(Keywords::INDESTRUCTIBLE),
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
                    Keywords::FLYING
                        .union(Keywords::HASTE)
                        .union(Keywords::TRAMPLE)
                        .union(Keywords::INDESTRUCTIBLE),
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: None,
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: true,
                            subtype: None,
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: true,
                        },
                        source_only: false,
                        own_turn: false,
                        keywords: Keywords::FLYING
                            .union(Keywords::HASTE)
                            .union(Keywords::TRAMPLE)
                            .union(Keywords::INDESTRUCTIBLE),
                    },
                )],
            },
        ],
        keyword_grants: &[KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: true,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords::FLYING
                .union(Keywords::HASTE)
                .union(Keywords::TRAMPLE)
                .union(Keywords::INDESTRUCTIBLE),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
