// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::PlayLandFromLibraryTop,
                StaticAbilityFragment::CastFromLibraryTop(CastFilter::Subtype("Bird")),
            ],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::ControlledLandOrSubtypeEntry("Bird"),
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
