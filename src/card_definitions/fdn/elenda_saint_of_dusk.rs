// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
const ABOVE: StatValue = StatValue::ControllerLifeAboveStarting { minimum: 1 };
const TEN_ABOVE: StatValue = StatValue::ControllerLifeAboveStarting { minimum: 10 };
const FIRST_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: ABOVE,
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const SECOND_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: TEN_ABOVE,
    stats: Stats {
        power: 5,
        toughness: 5,
    },
};
const MENACE: KeywordGrant = KeywordGrant {
    condition: Some(ABOVE),
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
    keywords: Keywords::MENACE,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::LIFELINK,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::LIFELINK)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::QualifiedHexproof(
                SourceCharacteristicFilter::Instant,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::KeywordGrant(0, MENACE),
                StaticAbilityFragment::ScalingModifier(0, FIRST_BONUS),
                StaticAbilityFragment::ScalingModifier(1, SECOND_BONUS),
            ],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[FIRST_BONUS, SECOND_BONUS],
        keyword_grants: &[MENACE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
