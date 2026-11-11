// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 3,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::ExcludingKinds(2),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Hand,
            },
            look: LibraryLook::Fixed(3),
            destination: LibraryPickDestination::ExilePlotted,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
