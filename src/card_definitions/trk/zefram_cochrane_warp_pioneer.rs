// Official TRK release notes reviewed; the pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastFromLibraryTopOnceEachOwnTurn(
                CastFilter::Artifact,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
                subtype: "Spacecraft",
                kind: CounterKind::Charge,
                amount: 2,
            }],
        },
    ],
    ..CardRules::EMPTY
};
