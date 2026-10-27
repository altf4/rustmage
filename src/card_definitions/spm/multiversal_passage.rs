// Candidate: gated pending review and full rules validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::PayLife(2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenBasicLandType],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::BasicLandType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
