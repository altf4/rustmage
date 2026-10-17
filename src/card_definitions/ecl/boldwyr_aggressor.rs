// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
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
    keywords: Keywords(1024),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1024))],
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
                        subtype: Some("Giant"),
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords(1024),
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1024))],
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
                            subtype: Some("Giant"),
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        source_only: false,
                        own_turn: false,
                        keywords: Keywords(1024),
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
                subtype: Some("Giant"),
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords(1024),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
