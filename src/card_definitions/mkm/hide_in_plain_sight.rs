pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(2),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::Cloak,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
