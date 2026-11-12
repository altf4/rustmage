// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(3),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }))),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
