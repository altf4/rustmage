const SPEC: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::BattleProtectedByOpponent,
    kind: Some(CounterKind::Defense),
    factor: 2,
    add: 0,
    effects_only: false,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, SPEC)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        replacement_effects: &[SPEC],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
