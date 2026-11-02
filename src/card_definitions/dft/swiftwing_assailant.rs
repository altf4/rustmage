// Direct typed behavior from the complete pinned Oracle and XMage sources.
const MAX_SPEED: StatValue = StatValue::AtLeast {
    value: &StatValue::ControllerSpeed,
    minimum: 4,
};
const BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: MAX_SPEED,
    stats: Stats {
        power: 0,
        toughness: 1,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::SourceConditionalKeywords {
                    condition: MAX_SPEED,
                    keywords: Keywords::VIGILANCE,
                },
                StaticAbilityFragment::ScalingModifier(0, BOOST),
            ],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
