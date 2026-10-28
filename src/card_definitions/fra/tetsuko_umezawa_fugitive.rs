// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(1),
        ],
    }],
    ..CardRules::EMPTY
};
