// A command-zone continuous effect reads this emblem's own frozen chosen type.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
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
        StaticAbilityFragment::ChosenCreatureKeywords(
            Keywords::VIGILANCE.union(Keywords::HEXPROOF),
        ),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
