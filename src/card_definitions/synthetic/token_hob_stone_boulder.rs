const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::DEFENDER,
    static_groups: GROUPS,
    deck_limit: None,
    ..CardRules::EMPTY
};
