// Complete pinned Oracle and XMage selection, shuffle and life-gain obligations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 20,
                    count: LibrarySelectionCount::Fixed(20),
                    filter: LibrarySelectionFilter::Kinds(2),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::Shuffle,
                },
                look: LibraryLook::Fixed(20),
                destination: LibraryPickDestination::BattlefieldTapped,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::GainLife(PlayerSet::Controller, 8),
        ],
    ),
    ..CardRules::EMPTY
};
