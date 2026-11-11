pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::DistinctCardTypes(1 | 2),
                filter: LibrarySelectionFilter::Any,
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
