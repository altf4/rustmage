pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryToExileRemember(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::DistinctCardTypes(
                        1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256,
                    ),
                    filter: LibrarySelectionFilter::Any,
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::ChosenX,
                destination: LibraryPickDestination::Exile,
                public: true,
                maximum_mana_value: None,
            }),
            Instruction::CastOneRememberedExileFreeThenRestToHand { minimum: 4 },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
