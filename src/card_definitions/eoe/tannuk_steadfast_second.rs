// Direct typed behavior from the pinned Oracle and XMage sources.
const HASTE_GRANT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::HASTE,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    extras: &CardExtras {
        keyword_grants: &[HASTE_GRANT],
        ..CardExtras::EMPTY
    },
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(0, HASTE_GRANT)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::GrantHandWarp {
                filter: CastFilter::AnyOf(&[
                    CastFilter::Artifact,
                    CastFilter::Colors {
                        filter: &CastFilter::Creature,
                        mask: 8,
                    },
                ]),
                cost: ManaCost {
                    generic: 2,
                    colored: [0, 0, 0, 1, 0, 0],
                },
            }],
        },
    ],
    ..CardRules::EMPTY
};
