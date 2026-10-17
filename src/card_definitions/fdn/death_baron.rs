// Direct typed behavior; complete pinned Oracle and XMage source reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::DeathBaronLord],
    }],
    ..CardRules::EMPTY
};
