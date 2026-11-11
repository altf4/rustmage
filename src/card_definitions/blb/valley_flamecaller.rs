// Direct typed behavior from complete pinned Oracle and XMage sources.
const DAMAGE: ReplacementSpec = ReplacementSpec::ControlledSubtypeDamagePlus {
    subtypes: &["Lizard", "Mouse", "Otter", "Raccoon"],
    amount: 1,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, DAMAGE)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        replacement_effects: &[DAMAGE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
