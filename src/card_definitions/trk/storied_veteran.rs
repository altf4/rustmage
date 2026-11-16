// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::CANT_BLOCK,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfGraveyardCastIfWithCounter {
            condition: StatValue::Fixed(1),
            counter: CounterKind::Finality,
        }],
    }],
    ..CardRules::EMPTY
};
