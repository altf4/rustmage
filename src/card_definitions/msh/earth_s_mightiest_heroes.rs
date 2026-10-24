// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfTeamwork { skip: 2 },
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 8,
                    count: LibrarySelectionCount::Fixed(8),
                    filter: LibrarySelectionFilter::Kinds(1),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Fixed(8),
                destination: LibraryPickDestination::Battlefield,
                public: true,
                maximum_mana_value: None,
            }),
            Instruction::Skip(1),
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 8,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Fixed(8),
                destination: LibraryPickDestination::Battlefield,
                public: true,
                maximum_mana_value: None,
            }),
        ],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 5,
            paid_targets: None,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
