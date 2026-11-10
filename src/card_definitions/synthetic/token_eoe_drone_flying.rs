// DroneFlyingToken's restriction is evaluated against each prospective attacker.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CanBlockOnlyFlying],
    }],
    ..CardRules::EMPTY
};
