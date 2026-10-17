// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        chosen_entry_counters: Some(CounterKind::Fellowship),
        ..CardExtras::EMPTY
    },
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
            kind: EntryChoiceKind::CreatureType,
            other: false,
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            source_counter: Some(CounterKind::Fellowship),
        })],
    }],
    ..CardRules::EMPTY
};
