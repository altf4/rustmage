pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 6,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(6),
            destination: LibraryPickDestination::MayEnterBattlefield {
                maximum_mana_value: 2,
                keywords: Keywords::HASTE,
            },
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
