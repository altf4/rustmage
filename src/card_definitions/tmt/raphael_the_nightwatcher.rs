// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(
            0,
            KeywordGrant {
                condition: None,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                source_only: false,
                own_turn: false,
                keywords: Keywords::DOUBLE_STRIKE,
            },
        )],
    }],
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 1,
        }],
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: None,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: true,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords::DOUBLE_STRIKE,
                },
            )],
        }],
        keyword_grants: &[KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords::DOUBLE_STRIKE,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
