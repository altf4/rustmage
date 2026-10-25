// Whole Oracle: intrinsic improvise/flying and current-caster noncreature spell grant.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        has_improvise: true,
        ..CardExtras::EMPTY
    },
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SpellImprovise(
            CastFilter::Noncreature,
        )],
    }],
    ..CardRules::EMPTY
};
