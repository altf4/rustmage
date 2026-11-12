pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::ControllerTurn,
                keywords: Keywords::HEXPROOF,
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CannotBeBlockedByNamelessCreatures],
        },
    ],
    ..CardRules::EMPTY
};
