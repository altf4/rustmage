// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
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
        .union(Keywords::LIFELINK)
        .union(Keywords::FIRST_STRIKE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FIRST_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::LIFELINK)],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::StaticModifier(
                    0,
                    StaticModifier {
                        own_turn: false,
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: true,
                            subtype: Some("Angel"),
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        stats: Stats {
                            power: 1,
                            toughness: 1,
                        },
                    },
                ),
                StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: None,
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: true,
                            subtype: Some("Angel"),
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: false,
                        own_turn: false,
                        keywords: Keywords::LIFELINK,
                    },
                ),
            ],
        },
    ],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Angel"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::FIRST_STRIKE,
                )],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::LIFELINK)],
            },
            StaticAbilityGroup {
                line: 3,
                first_layer: AbilityLayer::Ability,
                fragments: &[
                    StaticAbilityFragment::StaticModifier(
                        0,
                        StaticModifier {
                            own_turn: false,
                            requires_life_gain: false,
                            filter: CreatureFilter {
                                relation: 0,
                                other: true,
                                subtype: Some("Angel"),
                                keyword: Keywords::NONE,
                                attacking: false,
                                tokens: None,
                                artifact: false,
                            },
                            stats: Stats {
                                power: 1,
                                toughness: 1,
                            },
                        },
                    ),
                    StaticAbilityFragment::KeywordGrant(
                        0,
                        KeywordGrant {
                            condition: None,
                            requires_life_gain: false,
                            filter: CreatureFilter {
                                relation: 0,
                                other: true,
                                subtype: Some("Angel"),
                                keyword: Keywords::NONE,
                                attacking: false,
                                tokens: None,
                                artifact: false,
                            },
                            source_only: false,
                            own_turn: false,
                            keywords: Keywords::LIFELINK,
                        },
                    ),
                ],
            },
        ],
        keyword_grants: &[KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Angel"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords::LIFELINK,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
