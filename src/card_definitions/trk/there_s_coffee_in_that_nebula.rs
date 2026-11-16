// Full Oracle and pinned XMage class reviewed; the filter follows Oracle's nonland permanent condition.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Subtype("Plains"),
                    LibrarySelectionFilter::AllOf(&[
                        LibrarySelectionFilter::NonlandPermanent,
                        LibrarySelectionFilter::ManaValueAtMost(3),
                    ]),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            generic: 2,
            colored: [1, 0, 0, 0, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
