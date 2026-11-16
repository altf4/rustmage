// XMage has no class at the pinned revision. The pinned Oracle and official
// release rules require every revealed permanent to enter tapped.
const LAND_NAMES: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Kinds(2),
                    LibrarySelectionFilter::NonlandPermanent,
                ]),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::Stat(LAND_NAMES)),
            destination: LibraryPickDestination::BattlefieldTapped,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
