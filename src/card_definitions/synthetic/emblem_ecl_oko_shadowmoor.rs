// Chosen-type command-zone emblem from the pinned XMage definition.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ChosenModifier(ChosenModifier {
                kind: EntryChoiceKind::CreatureType,
                other: false,
                stats: Stats {
                    power: 3,
                    toughness: 3,
                },
                source_counter: None,
            }),
            StaticAbilityFragment::ChosenTypeKeywords(
                Keywords::VIGILANCE.union(Keywords::HEXPROOF),
            ),
        ],
    }],
    ..CardRules::EMPTY
};
