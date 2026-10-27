// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 2,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::BottomAny,
        })],
    ),
    ..CardRules::EMPTY
};
