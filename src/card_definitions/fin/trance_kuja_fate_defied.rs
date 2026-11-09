const FLARE_STAR: ReplacementSpec = ReplacementSpec::DoubleControlledSubtypeDamage("Wizard");

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, FLARE_STAR)],
    }],
    extras: &CardExtras {
        replacement_effects: &[FLARE_STAR],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
