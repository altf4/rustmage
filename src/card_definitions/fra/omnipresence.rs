// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeHandCastAtMostControlledCreatures],
    }],
    ..CardRules::EMPTY
};
