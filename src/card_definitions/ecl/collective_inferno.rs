// Direct typed behavior from the complete pinned Oracle and XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, ReplacementSpec::DoubleControlledChosenTypeDamage)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        has_convoke: true,
        entry_choice: Some(EntryChoiceKind::CreatureType),
        static_ability_groups: GROUPS,
        replacement_effects: &[ReplacementSpec::DoubleControlledChosenTypeDamage],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
