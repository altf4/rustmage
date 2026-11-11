const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::CommittedCrimeThisTurn,
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
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
