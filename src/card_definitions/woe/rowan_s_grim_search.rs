// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBargained { skip: 1 },
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 4,
                    count: LibrarySelectionCount::Fixed(2),
                    filter: LibrarySelectionFilter::Any,
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Fixed(4),
                destination: LibraryPickDestination::Top,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::LoseLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
