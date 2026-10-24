// Each exact permanent grants one own-turn free hand instant/sorcery cast.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 7),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OwnTurnFreeHandInstantSorcery],
    }],
    ..CardRules::EMPTY
};
