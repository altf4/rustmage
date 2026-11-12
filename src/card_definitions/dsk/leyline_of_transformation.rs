// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenCreatureSubtypeAcrossZones],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        opening_hand_battlefield: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
