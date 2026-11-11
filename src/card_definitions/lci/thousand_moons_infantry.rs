const STATIC_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::UntapDuringOtherUntapSteps],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        static_ability_groups: STATIC_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
