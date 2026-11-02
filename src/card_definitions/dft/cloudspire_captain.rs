// Direct typed behavior from complete pinned Oracle and XMage sources.
const OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::CREW_SADDLE_PLUS_TWO,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::CreatureGroupBonus {
                filters: &[
                    CreatureFilter {
                        subtype: Some("Mount"),
                        ..OWN
                    },
                    CreatureFilter {
                        subtype: Some("Vehicle"),
                        ..OWN
                    },
                ],
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::CREW_SADDLE_PLUS_TWO,
            )],
        },
    ],
    ..CardRules::EMPTY
};
