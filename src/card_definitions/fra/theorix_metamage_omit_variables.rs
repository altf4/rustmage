// Direct typed rules from pinned Oracle and XMage; gated pending tests.
const THRESHOLD: StatValue = StatValue::AtLeast {
    value: &StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    minimum: 7,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceGraveyardBonus {
                kind: 0,
                minimum: 7,
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            },
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: THRESHOLD,
                keywords: Keywords::FLYING,
            },
        ],
    }],
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5451)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
