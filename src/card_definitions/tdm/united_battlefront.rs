// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(2),
                filter: LibrarySelectionFilter::AllOf(&[
                    LibrarySelectionFilter::Kinds(1 | 2 | 16 | 32 | 64 | 256),
                    LibrarySelectionFilter::ExcludingKinds(1 | 2),
                ]),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(7),
            destination: LibraryPickDestination::Battlefield,
            public: false,
            maximum_mana_value: Some(AmountValue::Fixed(3)),
        })],
    ),
    ..CardRules::EMPTY
};
