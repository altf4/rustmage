// Pinned Oracle and complete XMage class: other controlled creature entries get one counter.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledCreatureEntryCountersByStat {
                kind: CounterKind::PlusOne,
                amount: StatValue::Fixed(1),
            },
        ],
    }],
    ..CardRules::EMPTY
};
