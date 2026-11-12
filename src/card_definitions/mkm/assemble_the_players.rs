// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
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
                StaticAbilityFragment::CastCreaturePowerAtMostFromLibraryTopOnceEachTurn(2),
            ],
        },
    ],
    ..CardRules::EMPTY
};
