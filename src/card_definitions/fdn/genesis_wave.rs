// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(u16::MAX),
                filter: LibrarySelectionFilter::Kinds(1 | 2 | 16 | 32 | 64),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::ChosenX,
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: Some(AmountValue::ChosenX),
        })],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
