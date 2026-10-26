// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
            kind: EntryChoiceKind::CreatureType,
            other: false,
            stats: Stats {
                power: 2,
                toughness: 2,
            },
            source_counter: None,
        })],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        adventure_identity: Some("An Unexpected Party // At the Door"),
        adventure_spell: Some(Card(5408)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
