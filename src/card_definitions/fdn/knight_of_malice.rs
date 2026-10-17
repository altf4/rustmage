// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
const CONDITION: StatValue = StatValue::HasColor {
    filter: CountFilter {
        zone: CountZone::Battlefield,
        relation: -1,
        kind: 0,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    },
    color: ChoiceColor::White,
};
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: CONDITION,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FIRST_STRIKE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FIRST_STRIKE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::QualifiedHexproof(
                SourceCharacteristicFilter::Color(ChoiceColor::White),
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
