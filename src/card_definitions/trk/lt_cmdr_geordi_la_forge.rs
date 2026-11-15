// Oracle and official Star Trek release notes; pinned XMage class is absent.
const DOUBLE_CHARGE: ReplacementSpec = ReplacementSpec::ControlledArtifactCountersPlaced {
    kind: CounterKind::Charge,
    factor: 2,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE_CHARGE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE_CHARGE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
