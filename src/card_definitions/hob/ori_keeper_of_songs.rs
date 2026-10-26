// Complete printed obligations from pinned Oracle and XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::HasEnduringStory,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Storied],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::HasEnduringStory,
                keywords: Keywords::VIGILANCE,
            },
            StaticAbilityFragment::ScalingModifier(0, BONUS),
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: GROUPS,
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
