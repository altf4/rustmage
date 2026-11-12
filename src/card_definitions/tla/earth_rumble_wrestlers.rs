const LAND_OR_ENTRY: StatValue = StatValue::AtLeast {
    value: &StatValue::Sum(&[
        StatValue::CountAllKinds(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1 | 2,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        StatValue::LandsEnteredThisTurn,
    ]),
    minimum: 1,
};
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: LAND_OR_ENTRY,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: LAND_OR_ENTRY,
                keywords: Keywords::TRAMPLE,
            },
            StaticAbilityFragment::ScalingModifier(0, BONUS),
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
