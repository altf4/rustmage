// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(2),
                filter: LibrarySelectionFilter::Kinds(3),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::LandsTappedRestHand,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
