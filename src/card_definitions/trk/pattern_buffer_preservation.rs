// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 2,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::Graveyard,
        })],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            generic: 1,
            colored: [0, 0, 1, 0, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
