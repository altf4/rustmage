// Draft from pinned Oracle and XMage; gated pending full rules validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledInstantSorcerySplitSecond],
    }],
    ..CardRules::EMPTY
};
