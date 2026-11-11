// Direct typed behavior from the pinned Oracle and XMage sources.
const LOOK: LibrarySelectionEffect = LibrarySelectionEffect {
    spec: LibrarySelectionSpec {
        look: 20,
        count: LibrarySelectionCount::Fixed(2),
        filter: LibrarySelectionFilter::Kinds(1),
        optional: true,
        reveal: true,
        remainder: LibraryRemainder::Shuffle,
    },
    look: LibraryLook::Fixed(20),
    destination: LibraryPickDestination::Hand,
    public: false,
    maximum_mana_value: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBargained { skip: 2 },
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                destination: LibraryPickDestination::Battlefield,
                ..LOOK
            }),
            Instruction::Skip(1),
            Instruction::SelectLibraryTo(LOOK),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
