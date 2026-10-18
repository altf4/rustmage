// Direct typed Oracle behavior; source and dependencies pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeHandCast],
    }],
    ..CardRules::EMPTY
};
