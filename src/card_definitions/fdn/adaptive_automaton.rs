// Gated core fixture: ordinary typed obligations, not an admission declaration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ChosenCreatureType],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
                kind: EntryChoiceKind::CreatureType,
                other: true,
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
                source_counter: None,
            })],
        },
    ],
    ..CardRules::EMPTY
};
