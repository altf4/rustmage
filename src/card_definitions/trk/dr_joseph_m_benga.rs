// Pinned DrJosephMBenga.java and Oracle: both static abilities apply while
// this creature is on the battlefield. The replacement keeps counter kinds.
const DOUBLE: ReplacementSpec =
    ReplacementSpec::ControlledCreatureCountersPlacedByController { factor: 2 };

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::ControlledOtherCounteredCreatureKeywords(Keywords::HEXPROOF),
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
