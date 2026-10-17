// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
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
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: Some(StatValue::Not(&StatValue::SpellsCastThisTurn)),
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: true,
                    own_turn: false,
                    keywords: Keywords::HEXPROOF,
                },
            )],
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
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::KeywordGrant(
                    0,
                    KeywordGrant {
                        condition: Some(StatValue::Not(&StatValue::SpellsCastThisTurn)),
                        requires_life_gain: false,
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: true,
                        own_turn: false,
                        keywords: Keywords::HEXPROOF,
                    },
                )],
            },
        ],
        keyword_grants: &[KeywordGrant {
            condition: Some(StatValue::Not(&StatValue::SpellsCastThisTurn)),
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: true,
            own_turn: false,
            keywords: Keywords::HEXPROOF,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
