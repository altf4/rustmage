// Direct typed Oracle behavior; full pinned XMage source and dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardPermanentTypes],
    }],
    ..CardRules::EMPTY
};
